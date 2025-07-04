"""
Server thread implementation for running FastAPI in tests.

This module provides a thread-based server runner for FastAPI applications
that can be used in tests without blocking the test execution.
"""

import threading
import time
import logging
from typing import Optional, Callable, Any

import uvicorn
from fastapi import FastAPI

logger = logging.getLogger(__name__)


class ServerThread(threading.Thread):
    """
    Thread class for running a uvicorn server in tests.
    
    This class starts a uvicorn server in a separate thread,
    allowing tests to run concurrently with the server.
    """
    
    def __init__(self, app: FastAPI, host: str = "127.0.0.1", port: int = 8000):
        """
        Initialize the server thread.
        
        Args:
            app: The FastAPI application to run
            host: Host to bind to
            port: Port to bind to
        """
        super().__init__(daemon=True)
        self.app = app
        self.host = host
        self.port = port
        self.started = threading.Event()
        self.ready = threading.Event()
        self.should_exit = threading.Event()
        self.server: Optional[uvicorn.Server] = None
        self._exception: Optional[Exception] = None
    
    def run(self):
        """Run the server in this thread."""
        try:
            # Configure the server
            config = uvicorn.Config(
                self.app,
                host=self.host,
                port=self.port,
                log_level="error"
            )
            
            # Create the server
            self.server = uvicorn.Server(config=config)
            
            # Override server's install_signal_handlers to not interfere with tests
            self.server.install_signal_handlers = lambda: None
            
            # Set the started event to indicate the server is about to start
            self.started.set()
            
            # Run the server until should_exit is set
            self.server.run()
        except Exception as e:
            logger.exception("Server thread encountered an error")
            self._exception = e
        finally:
            logger.info("Server thread exiting")
    
    def start_server(self, timeout: float = 5.0) -> bool:
        """
        Start the server and wait until it's ready.
        
        Args:
            timeout: Maximum seconds to wait for server to start
            
        Returns:
            True if server started successfully, False otherwise
            
        Raises:
            RuntimeError: If the server thread encountered an exception
        """
        # Start the thread
        self.start()
        
        # Wait for server to indicate it has started
        if not self.started.wait(timeout=timeout):
            return False
        
        # Short sleep to allow server to initialize
        time.sleep(0.5)
        
        # Check for exceptions
        if self._exception:
            raise RuntimeError(f"Server failed to start: {self._exception}")
        
        return True
    
    def stop_server(self, timeout: float = 5.0) -> bool:
        """
        Stop the server gracefully.
        
        Args:
            timeout: Maximum seconds to wait for server to stop
            
        Returns:
            True if server stopped successfully, False otherwise
        """
        if self.server:
            # Signal server to exit
            self.should_exit.set()
            if self.server.should_exit is not None:
                self.server.should_exit = True
            
            # Wait for thread to terminate
            self.join(timeout=timeout)
            return not self.is_alive()
        
        return True


def start_server_in_thread(
    app: FastAPI,
    host: str = "127.0.0.1",
    port: int = 8000,
    timeout: float = 5.0
) -> tuple[ServerThread, str]:
    """
    Start a FastAPI server in a background thread.
    
    Args:
        app: The FastAPI application to run
        host: Host to bind to
        port: Port to bind to
        timeout: Timeout for server startup
        
    Returns:
        Tuple of (ServerThread, server_url)
        
    Raises:
        RuntimeError: If server fails to start
    """
    # Create and start the server thread
    server_thread = ServerThread(app, host, port)
    
    if not server_thread.start_server(timeout=timeout):
        raise RuntimeError("Server failed to start within timeout period")
    
    # Return both the thread and the URL
    server_url = f"http://{host}:{port}"
    
    logger.info(f"Server started at {server_url}")
    return server_thread, server_url