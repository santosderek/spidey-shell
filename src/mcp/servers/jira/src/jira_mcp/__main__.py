import sys
import os
import argparse
import logging

from jira_mcp.server import mcp
from jira_mcp.settings import settings


def setup_logging(debug_mode=False):
    """Configure logging for the application."""
    log_level = logging.DEBUG if debug_mode else logging.INFO
    logging.basicConfig(
        level=log_level,
        format="%(asctime)s - [%(name)s] - %(levelname)s - %(message)s",
        stream=sys.stderr,  # Ensure logging goes to stderr
    )
    # Set the root logger level
    logging.getLogger().setLevel(log_level)


def main():
    """Parse arguments and run the MCP server."""
    parser = argparse.ArgumentParser(description="Jira MCP Server")
    parser.add_argument(
        "--debug", 
        action="store_true", 
        help="Enable debug logging"
    )
    parser.add_argument(
        "--transport",
        choices=["stdio", "streamable-http"],
        default="stdio",
        help="Transport mechanism for MCP communication (default: stdio)",
    )
    args = parser.parse_args()

    # Configure logging
    setup_logging(args.debug or settings.debug)
    
    # The main server entry point
    try:
        mcp.run(transport=args.transport)
    except Exception as e:
        logging.error(f"Failed to start MCP server: {e}")
        return 1
        
    return 0


if __name__ == "__main__":
    sys.exit(main())

