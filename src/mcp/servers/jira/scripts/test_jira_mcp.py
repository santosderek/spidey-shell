#!/usr/bin/env python3
"""
Interactive testing script for jira_mcp stdio transport.

This script provides a command-line interface for testing the
jira_mcp package with stdio transport. It allows sending commands
to the MCP server and displaying the responses in a user-friendly way.
"""

import argparse
import json
import os
import subprocess
import sys
import threading
import time
import uuid
from pathlib import Path

# Default values
DEFAULT_RECENT_DAYS = 30


def format_output(output, debug=False):
    """Format output for display in the terminal."""
    try:
        # Try to parse as JSON
        data = json.loads(output)
        
        # If in debug mode, show the raw JSON response
        if debug:
            print("\nRaw Response:")
            print(json.dumps(data, indent=2))
        
        # Check if this is an error response
        if "error" in data:
            error = data["error"]
            print(f"\n❌ Error: {error.get('message', 'Unknown error')}")
            if "traceback" in error and debug:
                print("\nTraceback:")
                print(error["traceback"])
            return
        
        # Check if this is a result response with markdown
        if "result" in data:
            result = data["result"]
            
            # Print a summary of the result
            if "message" in result:
                print(f"\n✅ {result['message']}")
            
            # If there's markdown, display it
            if "markdown" in result:
                print("\nOutput:")
                print(result["markdown"])
                
            # If there's a count, show it
            if "count" in result:
                count = result["count"]
                if count == 0:
                    print("\nNo items found.")
                elif count == 1:
                    print("\n1 item found.")
                else:
                    print(f"\n{count} items found.")
    except json.JSONDecodeError:
        # If it's not valid JSON, print the raw output
        print(output)


def send_command(proc, tool_name, params=None):
    """
    Send a command to the MCP server.
    
    Args:
        proc: The subprocess instance
        tool_name: The name of the tool to call
        params: Parameters for the tool
    
    Returns:
        The response from the server
    """
    request_id = str(uuid.uuid4())
    request = {
        "jsonrpc": "2.0",
        "id": request_id,
        "method": "tools/call",
        "params": {
            "name": tool_name,
            "args": params or {}
        }
    }
    
    # Send the request
    try:
        print(f"Sending request to tool: {tool_name}")
        proc.stdin.write(json.dumps(request) + "\n")
        proc.stdin.flush()
        
        # Read responses until we find ours
        for _ in range(10):  # Timeout after 10 attempts
            line = proc.stdout.readline()
            if not line:
                return {"error": "No response from server"}
            
            try:
                response = json.loads(line)
                if "id" in response and response["id"] == request_id:
                    return response
            except json.JSONDecodeError:
                print(f"Warning: Received non-JSON data: {line.strip()}")
        
        return {"error": "No matching response received after timeout"}
    except BrokenPipeError:
        return {"error": "Connection to server lost"}


def parse_command(cmd):
    """
    Parse a user command into a tool name and parameters.
    
    Args:
        cmd: The command string entered by the user
    
    Returns:
        A tuple of (tool_name, parameters)
    """
    parts = cmd.strip().split()
    if not parts:
        return None, None
    
    command = parts[0].lower()
    
    # Handle simple commands
    if command == "help":
        return None, None  # Help is handled separately
    
    if command == "tickets":
        # Handle tickets commands
        if len(parts) > 1 and parts[1].lower() == "recent":
            # Handle "tickets recent [days]"
            days = int(parts[2]) if len(parts) > 2 else DEFAULT_RECENT_DAYS
            return "list_tickets", {"filter": "recent", "days": days}
        else:
            # Default to assigned tickets
            return "list_tickets", {"filter": "assigned"}
    
    if command == "ticket":
        # Handle "ticket <ticket_id>"
        if len(parts) < 2:
            print("Error: Missing ticket ID")
            return None, None
        return "view_ticket", {"ticket_id": parts[1]}
    
    if command == "comments":
        # Handle "comments <ticket_id>"
        if len(parts) < 2:
            print("Error: Missing ticket ID")
            return None, None
        return "show_ticket_comments", {"ticket_id": parts[1]}
    
    if command == "raw":
        # Handle raw commands: raw <tool_name> <json_args>
        if len(parts) < 2:
            print("Error: Missing tool name")
            return None, None
        
        tool_name = parts[1]
        
        # Parse JSON args if provided
        if len(parts) > 2:
            args_str = " ".join(parts[2:])
            try:
                args = json.loads(args_str)
            except json.JSONDecodeError:
                print("Error: Invalid JSON arguments")
                return None, None
        else:
            args = {}
            
        return tool_name, args
    
    # If nothing matched, return the command as is (might be a direct tool name)
    return command, {}


def print_help():
    """Print help information for the interactive testing script."""
    print("\nAvailable commands:")
    print("  help                    - Show this help message")
    print("  tickets                 - List assigned tickets")
    print("  tickets recent [days]   - List recently updated tickets (default: 30 days)")
    print("  ticket <ticket_id>      - View details of a specific ticket")
    print("  comments <ticket_id>    - View comments for a specific ticket")
    print("  raw <tool> <args_json>  - Send a raw command to the server")
    print("  exit, quit              - Exit the interactive session")


def main():
    """Run the interactive testing script."""
    parser = argparse.ArgumentParser(
        description="Interactive testing script for jira_mcp stdio transport"
    )
    parser.add_argument(
        "--debug", 
        action="store_true", 
        help="Enable debug mode"
    )
    parser.add_argument(
        "--mock", 
        action="store_true", 
        help="Use mock server for testing"
    )
    args = parser.parse_args()
    
    # Determine project root path
    project_root = Path(__file__).parent.parent
    
    # Prepare environment variables
    env = os.environ.copy()
    mock_server_proc = None
    mock_server_port = 8000
    
    if args.mock:
        print("Starting mock server...")
        # Find a free port
        import socket
        s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        s.bind(('', 0))
        mock_server_port = s.getsockname()[1]
        s.close()
        
        # Start mock server
        mock_server_proc = subprocess.Popen(
            ["python", "-c", 
             "from tests.mock_server.server import app; "
             "import uvicorn; "
             f"uvicorn.run(app, host='127.0.0.1', port={mock_server_port})"],
            stdout=subprocess.PIPE if not args.debug else None,
            stderr=subprocess.PIPE if not args.debug else None,
            cwd=project_root
        )
        
        # Allow server to start
        print(f"Started mock server on port {mock_server_port}")
        time.sleep(2)
        
        # Configure environment to use the mock server
        env["JIRA_MCP_JIRA_URL"] = f"http://127.0.0.1:{mock_server_port}"
        env["JIRA_MCP_JIRA_USERNAME"] = "test_user"
        env["JIRA_MCP_JIRA_API_TOKEN"] = "test_password"
    
    if args.debug:
        env["JIRA_MCP_DEBUG"] = "true"
    
    # Prepare the command to run jira_mcp (explicitly with stdio transport)
    cmd = ["python", "-m", "jira_mcp", "--transport", "stdio"]
    if args.debug:
        cmd.append("--debug")
    
    # Start the jira_mcp process with stdio transport
    print("Starting jira_mcp server with stdio transport...")
    proc = subprocess.Popen(
        cmd,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        cwd=project_root,
        env=env,
        bufsize=1,  # Line buffered
    )
    
    # Start a thread to monitor stderr
    def monitor_stderr():
        for line in iter(proc.stderr.readline, ""):
            if args.debug:
                print(f"[stderr] {line.strip()}", file=sys.stderr)
    
    stderr_thread = threading.Thread(target=monitor_stderr, daemon=True)
    stderr_thread.start()
    
    # Wait for server to start
    print("Waiting for server to initialize...")
    time.sleep(2)
    
    # Initialize the MCP server
    print("Initializing MCP server...")
    init_request = {
        "jsonrpc": "2.0",
        "id": "init",
        "method": "initialize",
        "params": {
            "protocolVersion": "0.5.0",
            "capabilities": {},
            "clientInfo": {
                "name": "test_jira_mcp.py",
                "version": "1.0.0"
            }
        }
    }
    proc.stdin.write(json.dumps(init_request) + "\n")
    proc.stdin.flush()
    
    # Send initialized notification (this is a notification, not a request)
    print("Sending initialized notification...")
    initialized_notification = {
        "jsonrpc": "2.0",
        "method": "notifications/initialized",
        "params": {}
    }
    proc.stdin.write(json.dumps(initialized_notification) + "\n")
    proc.stdin.flush()
    
    # Note: initialized is a notification, so the server doesn't send a response
    # We wait a short time for the notification to be processed
    print("Waiting for notification to be processed...")
    time.sleep(1)
    
    # Read the initialization response
    init_response = None
    for _ in range(10):  # Timeout after 10 attempts
        line = proc.stdout.readline()
        if not line:
            print("Error: No initialization response from server")
            break
            
        try:
            response = json.loads(line)
            if "id" in response and response["id"] == "init":
                init_response = response
                break
        except json.JSONDecodeError:
            print(f"Warning: Received non-JSON data: {line.strip()}")
    
    if init_response:
        if "result" in init_response:
            print("MCP server initialized successfully")
        else:
            print("Error initializing MCP server:", init_response.get("error", {}).get("message", "Unknown error"))
    else:
        print("Error: No initialization response received")
    
    # Print welcome message
    print("\n========================================")
    print("Interactive jira_mcp stdio testing script")
    print("========================================\n")
    print("Type 'help' for available commands")
    print("Type 'exit' or 'quit' to exit\n")
    
    # Main interaction loop
    try:
        while True:
            try:
                cmd = input("> ")
            except EOFError:
                break
                
            if cmd.lower() in ("exit", "quit"):
                break
                
            if cmd.lower() == "help":
                print_help()
                continue
                
            # Parse and execute the command
            tool_name, params = parse_command(cmd)
            if tool_name:
                response = send_command(proc, tool_name, params)
                format_output(json.dumps(response), args.debug)
    except KeyboardInterrupt:
        print("\nInterrupted by user")
    finally:
        # Clean up
        print("\nShutting down...")
        proc.terminate()
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()
            proc.wait()
            
        # Clean up mock server if it was started
        if mock_server_proc:
            print("Stopping mock server...")
            mock_server_proc.terminate()
            try:
                mock_server_proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                mock_server_proc.kill()
                mock_server_proc.wait()


if __name__ == "__main__":
    main()