"""
Jira Mock Server Manager.

This module provides functions to manage the Jira mock server,
integrating it with the MCP system.
"""
import logging
import threading
from typing import Dict, Any, Optional

from . import mcp
from jira_mcp.settings import settings

logger = logging.getLogger(__name__)

# Global server instance
_server_thread = None
_server_running = False


@mcp.tool()
def start_mock_server(args: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
    """
    Start a local Jira mock server.
    
    Args:
        args: Dictionary containing optional parameters:
              - host: Host address to bind to (default: 0.0.0.0)
              - port: Port to listen on (default: 8000)
    
    Returns:
        Dictionary with server status information.
    """
    global _server_thread, _server_running
    
    if _server_running:
        return {
            "message": "Jira mock server is already running",
            "status": "running",
            "markdown": "⚠️ Jira mock server is already running."
        }
    
    args = args or {}
    host = args.get("host", "0.0.0.0")
    port = int(args.get("port", settings.mock_server_port))
    
    # Import the mock server from tests directory
    from tests.mock_server.jira_mock import run_server
    
    def _run_server():
        try:
            run_server(host=host, port=port, debug=settings.debug)
        except Exception as e:
            logger.error(f"Error in mock server: {e}")
            global _server_running
            _server_running = False
    
    try:
        _server_thread = threading.Thread(target=_run_server, daemon=True)
        _server_thread.start()
        _server_running = True
        
        server_url = f"http://localhost:{port}"
        
        return {
            "message": f"Jira mock server started on {host}:{port}",
            "status": "running",
            "url": server_url,
            "markdown": f"""
### Jira Mock Server Started

Server is now running at [http://localhost:{port}](http://localhost:{port}).

Available endpoints:
- Server Info: [/rest/api/2/serverInfo](http://localhost:{port}/rest/api/2/serverInfo)
- Current User: [/rest/api/2/myself](http://localhost:{port}/rest/api/2/myself)
- Projects: [/rest/api/2/project](http://localhost:{port}/rest/api/2/project)
- Fields: [/rest/api/2/field](http://localhost:{port}/rest/api/2/field)

To configure the Python JIRA client to use this mock server:
```python
from jira import JIRA

jira = JIRA(server='http://localhost:{port}', basic_auth=('mock_user', 'mock_pass'))
```
"""
        }
    except Exception as e:
        logger.exception("Failed to start Jira mock server")
        return {
            "message": f"Failed to start Jira mock server: {e}",
            "status": "error",
            "markdown": f"❌ Failed to start Jira mock server: {str(e)}"
        }


@mcp.tool()
def mock_server_status(args: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
    """
    Check status of the Jira mock server.
    
    Returns:
        Dictionary with server status information.
    """
    global _server_running
    
    status = "running" if _server_running else "stopped"
    port = settings.mock_server_port
    
    if _server_running:
        return {
            "status": status,
            "url": f"http://localhost:{port}",
            "message": f"Jira mock server is {status}",
            "markdown": f"""
### Jira Mock Server Status: Running

Server is currently running at [http://localhost:{port}](http://localhost:{port}).

Key endpoints:
- Server Info: [/rest/api/2/serverInfo](http://localhost:{port}/rest/api/2/serverInfo)
- Health Check: [/health](http://localhost:{port}/health)
"""
        }
    else:
        return {
            "status": status,
            "message": f"Jira mock server is {status}",
            "markdown": "### Jira Mock Server Status: Stopped\n\nServer is not currently running."
        }