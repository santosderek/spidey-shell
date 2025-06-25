use std::error::Error;
use std::io;
use std::path::PathBuf;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

// Import from library
use spidey_shell::app::events;
use spidey_shell::app::state::AppState;
use spidey_shell::mcp::MCPServerManager;

// Library imports
use spidey_shell::credentials::CredentialManager;
use spidey_shell::openai::{AzureOpenAIClient, AzureOpenAIConfig};
use spidey_shell::persistence::config::AppConfig;
use spidey_shell::persistence::ConversationStore;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();

    // Initialize credential manager
    let credential_manager = CredentialManager::new();

    // Check if we're running in a proper terminal
    let has_tty =
        crossterm::tty::IsTty::is_tty(&io::stdout()) && crossterm::tty::IsTty::is_tty(&io::stdin());

    if !has_tty {
        eprintln!("Error: This application requires a TTY terminal to run.");
        eprintln!("Please run this application in a proper terminal environment.");
        eprintln!("If you're using VS Code, try running it in the integrated terminal.");
        eprintln!("If you're using a CI/automated environment, this application is not suitable for that context.");
        std::process::exit(1);
    }

    // Load config and store
    let mut config = AppConfig::load().unwrap_or_default();
    let db_path = config.db_path.clone();
    let store = ConversationStore::new(db_path)?;

    // Set up OpenAI client
    let oaiclient = setup_openai_client(&credential_manager)?;

    // Set up MCP server manager
    let mut mcp_manager = setup_mcp_manager(&credential_manager)?;

    // Set up terminal
    let mut terminal = setup_terminal()?;

    // Initialize app state
    let mut app = AppState::new(&mcp_manager, &store);

    // Run the app
    let res = events::run_app(
        &mut terminal,
        &mut app,
        &mut config,
        &store,
        &oaiclient,
        &mut mcp_manager,
    )
    .await;

    // Cleanup terminal on exit
    cleanup_terminal(terminal)?;

    if let Err(err) = res {
        println!("Error: {}", err);
    }

    Ok(())
}

/// Set up the OpenAI client with credentials
fn setup_openai_client(
    credential_manager: &CredentialManager,
) -> Result<AzureOpenAIClient, Box<dyn Error>> {
    // Get credentials from credential manager
    let api_key = credential_manager
        .get("AZURE_OPENAI_API_KEY")
        .or_else(|| credential_manager.get("AZURE_OPENAI_KEY"))
        .unwrap_or("");

    let api_base = credential_manager
        .get("AZURE_OPENAI_ENDPOINT")
        .unwrap_or("");
    let deployment_id = credential_manager
        .get("AZURE_OPENAI_DEPLOYMENT_ID")
        .unwrap_or("");
    let api_version = "2024-02-15-preview"; // Fixed API version that works with our endpoint
    let model = credential_manager
        .get("AZURE_OPENAI_MODEL")
        .unwrap_or("gpt-4.1");

    let oaiconfig = AzureOpenAIConfig {
        api_key: api_key.to_string(),
        api_base: api_base.to_string(),
        deployment_id: deployment_id.to_string(),
        api_version: api_version.to_string(),
        model: model.to_string(),
    };

    // Check if we have required credentials
    if api_key.is_empty() || api_base.is_empty() || deployment_id.is_empty() {
        eprintln!("Warning: Missing required Azure OpenAI credentials.");
        eprintln!("Please set the following variables in your environment:");
        eprintln!("- AZURE_OPENAI_API_KEY (or AZURE_OPENAI_KEY)");
        eprintln!("- AZURE_OPENAI_ENDPOINT");
        eprintln!("- AZURE_OPENAI_DEPLOYMENT_ID");
    }

    Ok(AzureOpenAIClient::new(oaiconfig))
}

/// Set up the MCP server manager with automatic server discovery
fn setup_mcp_manager(
    _credential_manager: &CredentialManager,
) -> Result<Option<MCPServerManager>, Box<dyn Error>> {
    // Set up MCP server base directory
    let mcp_base_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config/spidey-shell/mcp_servers");

    match MCPServerManager::new(mcp_base_dir.clone()) {
        Ok(mut manager) => {
            println!("Initializing MCP server manager...");

            // The manager automatically discovers and loads all MCP servers during initialization
            // via the load_all() method, which uses MCPProjectDiscovery to scan for Python projects
            // and creates server configurations for them.

            // Get the list of discovered servers
            let server_names = manager.get_server_names();
            if server_names.is_empty() {
                println!("No MCP servers discovered. You can add MCP Python projects to:");
                println!("  {}", mcp_base_dir.join("projects").display());
            } else {
                println!("Discovered {} MCP server(s):", server_names.len());
                for name in &server_names {
                    if let Some(config) = manager.get_server(name) {
                        println!(
                            "  - {} ({})",
                            name,
                            config.description.as_deref().unwrap_or("No description")
                        );
                        if !config.capabilities.is_empty() {
                            println!("    Capabilities: {}", config.capabilities.join(", "));
                        }
                    }
                }
            }

            // Start all Python MCP processes
            match manager.start_all_python_processes() {
                Ok(started) => {
                    if started > 0 {
                        println!("Started {} Python MCP process(es)", started);
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to start some Python MCP processes: {}", e);
                }
            }

            Ok(Some(manager))
        }
        Err(e) => {
            eprintln!("Warning: Failed to initialize MCP server manager: {}", e);
            eprintln!("MCP server functionality will be disabled.");
            Ok(None)
        }
    }
}

/// Set up the terminal interface
fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>, Box<dyn Error>> {
    enable_raw_mode().map_err(|e| format!("Failed to enable raw mode: {}", e))?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .map_err(|e| format!("Failed to setup terminal: {}", e))?;

    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend).map_err(|e| format!("Failed to create terminal: {}", e).into())
}

/// Clean up the terminal on exit
fn cleanup_terminal(
    mut terminal: Terminal<CrosstermBackend<io::Stdout>>,
) -> Result<(), Box<dyn Error>> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}
