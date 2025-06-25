"""
MCP server implementation for Jira.

This module provides the MCP server implementation for Jira integration.
"""

import sys
import json
import argparse
import logging
from typing import Dict, Any

# Conditionally import dotenv for development environments
try:
    from dotenv import load_dotenv

    load_dotenv()
except ImportError:
    pass  # Skip if dotenv is not available

from jira_mcp.tools import MCP_TOOLS

logger = logging.getLogger(__name__)


def configure_logging(debug: bool = False) -> None:
    """Configure logging for the MCP server."""
    log_level = logging.DEBUG if debug else logging.INFO
    logging.basicConfig(
        level=log_level, format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
    )


def process_message(message: Dict[str, Any]) -> Dict[str, Any]:
    """Process an incoming MCP message and return a response."""
    try:
        command = message.get("command", "")
        logger.debug(f"Processing command: {command}")

        if command in MCP_TOOLS:
            tool_config = MCP_TOOLS[command]
            tool_func = tool_config["function"]
            return tool_func(message)
        elif command == "list-commands":
            return {
                "commands": [
                    {
                        "name": cmd,
                        "description": cfg["description"],
                        "args": cfg["args"],
                    }
                    for cmd, cfg in MCP_TOOLS.items()
                ]
            }
        else:
            return {
                "error": f"Unknown command: {command}",
                "available_commands": list(MCP_TOOLS.keys()) + ["list-commands"],
            }

    except Exception as e:
        logger.error(f"Error processing message: {e}")
        return {"error": f"Internal server error: {str(e)}"}


def main() -> int:
    """Run the Jira MCP server."""
    parser = argparse.ArgumentParser(description="Jira MCP Server")
    parser.add_argument("--debug", action="store_true", help="Enable debug logging")
    args = parser.parse_args()

    configure_logging(args.debug)
    logger.info("Starting Jira MCP server")

    # Process stdin messages
    for line in sys.stdin:
        try:
            # Parse incoming JSON message
            message = json.loads(line.strip())
            logger.debug(f"Received message: {message}")

            # Process the message
            response = process_message(message)

            # Send the response
            print(json.dumps(response), flush=True)
        except json.JSONDecodeError:
            logger.error("Invalid JSON received")
        except Exception as e:
            logger.error(f"Error processing message: {e}")
            print(
                json.dumps({"error": f"Internal server error: {str(e)}"}), flush=True
            )

    logger.info("Jira MCP server stopped")
    return 0


if __name__ == "__main__":
    sys.exit(main())