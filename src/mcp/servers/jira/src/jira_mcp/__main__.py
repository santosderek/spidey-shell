"""
Main entry point for the Jira MCP server.

This module allows the package to be run directly with `python -m jira_mcp`.
It supports two modes of operation:
1. Standard MCP mode (reading from stdin/stdout)
2. FastMCP web server mode (FastAPI HTTP server)
"""

import sys
import argparse
import logging

def main():
    """Parse arguments and run in the appropriate mode."""
    parser = argparse.ArgumentParser(description="Jira MCP Server")
    parser.add_argument("--mode", choices=["standard", "web"], default="standard",
                       help="Run mode: standard (stdin/stdout) or web (HTTP server)")
    parser.add_argument("--port", type=int, default=8000,
                       help="Port to use for web server mode (default: 8000)")
    parser.add_argument("--host", default="0.0.0.0",
                       help="Host to bind to in web server mode (default: 0.0.0.0)")
    parser.add_argument("--debug", action="store_true", help="Enable debug logging")
    args = parser.parse_args()

    # Configure logging
    log_level = logging.DEBUG if args.debug else logging.INFO
    logging.basicConfig(
        level=log_level, format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
    )
    
    if args.mode == "web":
        # Import and run the FastMCP server
        from jira_mcp.fastmcp_server import run_server
        run_server(host=args.host, port=args.port, log_level="debug" if args.debug else "info")
    else:
        # Run the standard MCP server
        from jira_mcp.server import main as standard_main
        return standard_main()

if __name__ == "__main__":
    sys.exit(main())