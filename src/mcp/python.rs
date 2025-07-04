use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

/// Status of a Python MCP subprocess
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MCPProcessStatus {
    /// Process is not running
    NotRunning,
    /// Process is starting up
    Starting,
    /// Process is running and ready to accept requests
    Running,
    /// Process is shutting down
    ShuttingDown,
    /// Process has failed
    Failed,
}

/// Configuration for a Python MCP subprocess
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPPythonConfig {
    /// Name of the MCP server
    pub name: String,
    /// Path to the project directory
    pub project_path: PathBuf,
    /// Package name to run with python -m
    pub package_name: String,
    /// Additional arguments to pass to the Python process
    pub args: Vec<String>,
    /// Environment variables to set for the Python process
    pub env: HashMap<String, String>,
    /// Timeout in seconds for process startup
    pub startup_timeout: u64,
    /// Port the server will listen on, if applicable
    pub port: Option<u16>,
}

impl MCPPythonConfig {
    /// Create a new Python MCP configuration
    pub fn new(name: &str, project_path: PathBuf, package_name: &str) -> Self {
        Self {
            name: name.to_string(),
            project_path,
            package_name: package_name.to_string(),
            args: Vec::new(),
            env: HashMap::new(),
            startup_timeout: 30,
            port: None,
        }
    }
}

/// Represents a Python MCP subprocess
pub struct MCPPythonProcess {
    /// Configuration for this process
    pub config: MCPPythonConfig,
    /// Current status of the process
    status: Arc<Mutex<MCPProcessStatus>>,
    /// Process handle
    process: Option<Child>,
    /// Last time the process was started
    start_time: Option<Instant>,
}

impl MCPPythonProcess {
    /// Create a new Python MCP process
    pub fn new(config: MCPPythonConfig) -> Self {
        Self {
            config,
            status: Arc::new(Mutex::new(MCPProcessStatus::NotRunning)),
            process: None,
            start_time: None,
        }
    }

    /// Start the Python subprocess
    pub fn start(&mut self) -> io::Result<()> {
        if let Some(proc) = self.process.as_mut() {
            // Check if process is still running
            match proc.try_wait() {
                Ok(Some(_)) => {
                    // Process has exited, we can restart it
                }
                Ok(None) => {
                    // Process is still running
                    return Ok(());
                }
                Err(e) => {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("Failed to check process status: {}", e),
                    ));
                }
            }
        }

        // Set status to starting
        {
            let mut status = self.status.lock().unwrap();
            *status = MCPProcessStatus::Starting;
        }

        // Prepare command
        let mut cmd = Command::new("uv");
        cmd.arg("pip")
            .arg("run")
            .arg("--")
            .arg("python")
            .arg("-m")
            .arg(&self.config.package_name)
            .current_dir(&self.config.project_path)
            .args(&self.config.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Add environment variables
        for (key, value) in &self.config.env {
            cmd.env(key, value);
        }

        // Start the process
        info!("Starting Python MCP process: {}", self.config.name);
        match cmd.spawn() {
            Ok(child) => {
                self.process = Some(child);
                self.start_time = Some(Instant::now());

                // Create a clone of the status Arc for the monitoring thread
                let status_clone = Arc::clone(&self.status);
                let name = self.config.name.clone();
                let timeout = self.config.startup_timeout;

                // Spawn a thread to monitor the output and set status to Running when ready
                if let Some(mut child) = self.process.take() {
                    if let (Some(stdout), Some(stderr)) = (child.stdout.take(), child.stderr.take())
                    {
                        // Create a struct to hold the child process in the spawned thread
                        struct ChildProcess {
                            child: Child,
                        }

                        let child_process = ChildProcess { child };

                        std::thread::spawn(move || {
                            let mut child_proc = child_process;
                            let start_time = Instant::now();
                            let reader = BufReader::new(stdout);

                            // Create another thread to monitor stderr
                            let stderr_name = name.clone();
                            let stderr_thread = std::thread::spawn(move || {
                                let stderr_reader = BufReader::new(stderr);
                                for line in stderr_reader.lines() {
                                    match line {
                                        Ok(line) => {
                                            warn!("[{} stderr] {}", stderr_name, line);
                                        }
                                        Err(e) => {
                                            error!(
                                                "Error reading stderr for {}: {}",
                                                stderr_name, e
                                            );
                                            break;
                                        }
                                    }
                                }
                            });

                            // Look for indication that the server is running
                            for line in reader.lines() {
                                match line {
                                    Ok(line) => {
                                        debug!("[{}] {}", name, line);
                                        // TODO: Update this condition based on your server's output
                                        if line.contains("Server started")
                                            || line.contains("Running on")
                                        {
                                            let mut status = status_clone.lock().unwrap();
                                            *status = MCPProcessStatus::Running;
                                            info!("Python MCP process {} is now running", name);
                                            break;
                                        }
                                    }
                                    Err(e) => {
                                        error!("Error reading stdout for {}: {}", name, e);
                                        let mut status = status_clone.lock().unwrap();
                                        *status = MCPProcessStatus::Failed;
                                        break;
                                    }
                                }

                                // Check if we've timed out
                                if start_time.elapsed().as_secs() > timeout {
                                    error!("Timeout waiting for {} to start", name);
                                    let mut status = status_clone.lock().unwrap();
                                    *status = MCPProcessStatus::Failed;
                                    break;
                                }
                            }

                            // Now just keep monitoring the process
                            match child_proc.child.wait() {
                                Ok(status) => {
                                    if !status.success() {
                                        error!(
                                            "Python MCP process {} exited with code: {:?}",
                                            name,
                                            status.code()
                                        );
                                        let mut status_guard = status_clone.lock().unwrap();
                                        *status_guard = MCPProcessStatus::Failed;
                                    } else {
                                        info!("Python MCP process {} exited normally", name);
                                        let mut status_guard = status_clone.lock().unwrap();
                                        *status_guard = MCPProcessStatus::NotRunning;
                                    }
                                }
                                Err(e) => {
                                    error!("Error waiting for Python MCP process {}: {}", name, e);
                                    let mut status_guard = status_clone.lock().unwrap();
                                    *status_guard = MCPProcessStatus::Failed;
                                }
                            }

                            // Wait for stderr thread to finish
                            let _ = stderr_thread.join();
                        });

                        // Create a new process instance to replace the one we took
                        let mut cmd = Command::new("uv");
                        cmd.arg("pip")
                            .arg("run")
                            .arg("--")
                            .arg("python")
                            .arg("-m")
                            .arg(&self.config.package_name)
                            .current_dir(&self.config.project_path)
                            .args(&self.config.args)
                            .stdin(Stdio::piped())
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped());

                        // Add environment variables
                        for (key, value) in &self.config.env {
                            cmd.env(key, value);
                        }

                        match cmd.spawn() {
                            Ok(new_child) => {
                                self.process = Some(new_child);
                            }
                            Err(e) => {
                                error!("Failed to create new process handle: {}", e);
                                let mut status = self.status.lock().unwrap();
                                *status = MCPProcessStatus::Failed;
                            }
                        }
                    } else {
                        // If we couldn't take stdout/stderr, put the child back
                        self.process = Some(child);
                        return Err(io::Error::new(
                            io::ErrorKind::Other,
                            "Failed to take stdout/stderr from child process",
                        ));
                    }
                }
                Ok(())
            }
            Err(e) => {
                error!(
                    "Failed to start Python MCP process {}: {}",
                    self.config.name, e
                );
                let mut status = self.status.lock().unwrap();
                *status = MCPProcessStatus::Failed;
                Err(e)
            }
        }
    }

    /// Stop the Python subprocess
    pub fn stop(&mut self) -> io::Result<()> {
        if let Some(mut proc) = self.process.take() {
            info!("Stopping Python MCP process: {}", self.config.name);

            {
                let mut status = self.status.lock().unwrap();
                *status = MCPProcessStatus::ShuttingDown;
            }

            // Try graceful shutdown by sending SIGTERM
            #[cfg(unix)]
            {
                let pid = proc.id();
                unsafe {
                    libc::kill(pid as i32, libc::SIGTERM);
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            #[cfg(windows)]
            {
                proc.kill()?;
            }

            // Give it a moment to shut down
            match proc.wait_timeout(Duration::from_secs(5))? {
                Some(_) => {
                    info!("Python MCP process {} stopped", self.config.name);
                }
                None => {
                    warn!(
                        "Python MCP process {} did not stop gracefully, forcing",
                        self.config.name
                    );
                    proc.kill()?;
                    proc.wait()?;
                }
            }

            {
                let mut status = self.status.lock().unwrap();
                *status = MCPProcessStatus::NotRunning;
            }

            self.start_time = None;
            Ok(())
        } else {
            // No process to stop
            let mut status = self.status.lock().unwrap();
            *status = MCPProcessStatus::NotRunning;
            Ok(())
        }
    }

    /// Send data to the Python subprocess
    pub fn send(&mut self, data: &str) -> io::Result<()> {
        if let Some(proc) = &mut self.process {
            if let Some(stdin) = proc.stdin.as_mut() {
                stdin.write_all(data.as_bytes())?;
                stdin.write_all(b"\n")?;
                stdin.flush()?;
                Ok(())
            } else {
                Err(io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "Process stdin is not available",
                ))
            }
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "Process is not running",
            ))
        }
    }

    /// Get the current process status
    pub fn status(&self) -> MCPProcessStatus {
        *self.status.lock().unwrap()
    }

    /// Check if the process is running and ready
    pub fn is_running(&self) -> bool {
        self.status() == MCPProcessStatus::Running
    }

    /// Check if the process has failed
    pub fn has_failed(&self) -> bool {
        self.status() == MCPProcessStatus::Failed
    }

    /// Restart the process if it has failed or is not running
    pub fn ensure_running(&mut self) -> io::Result<bool> {
        match self.status() {
            MCPProcessStatus::NotRunning | MCPProcessStatus::Failed => {
                self.stop()?; // Make sure it's fully stopped
                self.start()?;
                Ok(true) // Process was restarted
            }
            MCPProcessStatus::Running => Ok(false), // Already running
            MCPProcessStatus::Starting => {
                // Check if we've been starting for too long
                if let Some(start_time) = self.start_time {
                    if start_time.elapsed().as_secs() > self.config.startup_timeout {
                        warn!(
                            "Python MCP process {} has been starting for too long, restarting",
                            self.config.name
                        );
                        self.stop()?;
                        self.start()?;
                        Ok(true)
                    } else {
                        Ok(false) // Still starting
                    }
                } else {
                    // No start time, reset status and restart
                    self.stop()?;
                    self.start()?;
                    Ok(true)
                }
            }
            MCPProcessStatus::ShuttingDown => {
                // Wait for it to shut down, then restart
                let mut retry_count = 0;
                while self.status() == MCPProcessStatus::ShuttingDown && retry_count < 10 {
                    std::thread::sleep(Duration::from_millis(100));
                    retry_count += 1;
                }
                self.stop()?; // Make sure it's fully stopped
                self.start()?;
                Ok(true)
            }
        }
    }
}

impl Drop for MCPPythonProcess {
    fn drop(&mut self) {
        if let Err(e) = self.stop() {
            error!(
                "Error stopping Python MCP process {}: {}",
                self.config.name, e
            );
        }
    }
}

/// Error-safe extension for Child to add wait_timeout functionality
trait ChildExt {
    fn wait_timeout(&mut self, timeout: Duration) -> io::Result<Option<std::process::ExitStatus>>;
}

impl ChildExt for Child {
    fn wait_timeout(&mut self, timeout: Duration) -> io::Result<Option<std::process::ExitStatus>> {
        let start = Instant::now();
        loop {
            match self.try_wait()? {
                Some(status) => return Ok(Some(status)),
                None => {
                    if start.elapsed() >= timeout {
                        return Ok(None);
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
            }
        }
    }
}
