#!/usr/bin/env python3
"""
Test script for jira_mcp stdio transport.

This script automatically tests the jira_mcp package with stdio transport
and verifies that all tools work correctly.
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


def send_request(proc, method, params=None):
    """Send a request to the MCP server and return the response."""
    request_id = str(uuid.uuid4())
    # The correct format for tool calls
    # The args parameter contains the arguments directly
    request = {
        "jsonrpc": "2.0",
        "id": request_id,
        "method": "tools/call",
        "params": {
            "name": method,
            "args": params or {}
        }
    }
    
    print(f"Sending request: {method} with arguments: {json.dumps(params or {})}")
    print(f"Full request: {json.dumps(request, indent=2)}")
    
    # Send the request
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
    
    return {"error": "No matching response received"}


def main():
    """Run the test script."""
    parser = argparse.ArgumentParser(
        description="Test jira_mcp stdio transport"
    )
    parser.add_argument(
        "--debug", 
        action="store_true", 
        help="Enable debug mode"
    )
    args = parser.parse_args()
    
    # Determine project root path
    project_root = Path(__file__).parent.parent
    
    # Start mock server
    print("Starting mock server...")
    import socket
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    s.bind(('', 0))
    mock_server_port = s.getsockname()[1]
    s.close()
    
    mock_server_proc = subprocess.Popen(
        ["python", "-c", 
         "from tests.mock_server.server import app; "
         "import uvicorn; "
         f"uvicorn.run(app, host='127.0.0.1', port={mock_server_port})"],
        stdout=subprocess.PIPE if not args.debug else None,
        stderr=subprocess.PIPE if not args.debug else None,
        cwd=project_root
    )
    
    print(f"Started mock server on port {mock_server_port}")
    time.sleep(2)
    
    # Prepare environment variables
    env = os.environ.copy()
    env["JIRA_MCP_JIRA_URL"] = f"http://127.0.0.1:{mock_server_port}"
    env["JIRA_MCP_JIRA_USERNAME"] = "test_user"
    env["JIRA_MCP_JIRA_API_TOKEN"] = "test_password"
    
    if args.debug:
        env["JIRA_MCP_DEBUG"] = "true"
    
    # Start jira_mcp with stdio transport
    print("Starting jira_mcp with stdio transport...")
    cmd = ["python", "-m", "jira_mcp", "--transport", "stdio"]
    if args.debug:
        cmd.append("--debug")
        
    proc = subprocess.Popen(
        cmd,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        cwd=project_root,
        env=env,
        bufsize=1
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
                "name": "test_stdio.py",
                "version": "1.0.0"
            }
        }
    }
    print(f"Sending initialization request: {json.dumps(init_request)}")
    proc.stdin.write(json.dumps(init_request) + "\n")
    proc.stdin.flush()
    
    # Read the initialization response
    init_response = None
    for attempt in range(10):  # Timeout after 10 attempts
        print(f"Reading initialization response (attempt {attempt+1})...")
        line = proc.stdout.readline()
        print(f"Raw response: {line.strip()}")
        
        if not line:
            print("Error: No initialization response from server")
            break
            
        try:
            response = json.loads(line)
            print(f"Parsed response: {json.dumps(response, indent=2)}")
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
    
    # List available tools
    print("Listing available tools...")
    list_tools_request = {
        "jsonrpc": "2.0",
        "id": "list_tools",
        "method": "tools/list",
        "params": {}
    }
    proc.stdin.write(json.dumps(list_tools_request) + "\n")
    proc.stdin.flush()
    
    # Read the list tools response
    list_tools_response = None
    for _ in range(10):  # Timeout after 10 attempts
        line = proc.stdout.readline()
        if not line:
            print("Error: No list tools response from server")
            break
            
        try:
            response = json.loads(line)
            if "id" in response and response["id"] == "list_tools":
                list_tools_response = response
                break
        except json.JSONDecodeError:
            print(f"Warning: Received non-JSON data: {line.strip()}")
    
    if list_tools_response and "result" in list_tools_response:
        print("Available tools:")
        tools = list_tools_response["result"]["tools"]
        for tool in tools:
            print(f"- {tool['name']}")
    else:
        print("Error listing tools:", list_tools_response)
    
    # Tests to run
    tests = [
        ("list_tickets", {"filter": "assigned"}),
        ("list_tickets", {"filter": "recent", "days": 7}),
        ("view_ticket", {"ticket_id": "TEST-1"}),
        ("view_ticket", {"ticket_id": "NONEXISTENT-1"}),
        ("show_ticket_comments", {"ticket_id": "TEST-2"}),
        ("show_ticket_comments", {"ticket_id": "TEST-1"}),
        ("nonexistent_tool", {}),
        ("view_ticket", {})
    ]
    
    # Run tests
    success_count = 0
    failure_count = 0
    
    for method, params in tests:
        print(f"\n{'-' * 50}")
        print(f"Testing: {method}")
        print(f"{'-' * 50}")
        
        response = send_request(proc, method, params)
        
        # Print the response
        print(f"Response: {json.dumps(response, indent=2)}")
        
        # Verify the response
        is_success = False
        
        if method == "nonexistent_tool":
            if "result" in response and response["result"].get("isError") == True and "Unknown tool" in str(response["result"]):
                print("✅ Got expected error response for nonexistent tool")
                is_success = True
            else:
                print("❌ Did not get error for nonexistent tool")
        elif method == "view_ticket" and not params:
            if "result" in response and response["result"].get("isError") == True: 
                print("✅ Got expected error response for missing parameters")
                is_success = True
            else:
                print("❌ Did not get error for missing parameters")
        elif method == "view_ticket" and params.get("ticket_id") == "NONEXISTENT-1":
            if "result" in response and response["result"].get("isError") == True:
                response_str = str(response["result"])
                # Check for either validation errors or ticket not found errors
                if "validation error" in response_str or "Ticket not found" in response_str:
                    print("✅ Got expected error response for nonexistent ticket")
                    is_success = True
                else:
                    print("❌ Did not get expected error for nonexistent ticket")
            else:
                print("❌ Did not get error for nonexistent ticket")
        else:
            if "result" in response:
                print("✅ Got successful response")
                
                # Additional checks based on the tool
                if method == "list_tickets":
                    result_str = str(response["result"])
                    if "tickets" in result_str and "Found" in result_str:
                        print("✅ Got tickets list in response")
                        is_success = True
                    else:
                        print("❌ Missing or invalid tickets in response")
                elif method == "view_ticket":
                    result_str = str(response["result"])
                    if ("ticket" in result_str and params.get("ticket_id") in result_str) or \
                       response["result"].get("isError") == True:
                        print("✅ Got ticket details or expected error")
                        is_success = True
                    else:
                        print("❌ Missing ticket details in response")
                elif method == "show_ticket_comments":
                    result_str = str(response["result"])
                    if "comments" in result_str:
                        print("✅ Got comments in response")
                        is_success = True
                    else:
                        print("❌ Missing comments in response")
            else:
                print("❌ Missing result in response")
        
        if is_success:
            success_count += 1
        else:
            failure_count += 1
    
    # Print summary
    print(f"\n{'-' * 50}")
    print("Test Summary")
    print(f"{'-' * 50}")
    print(f"Total tests: {len(tests)}")
    print(f"Successes: {success_count}")
    print(f"Failures: {failure_count}")
    
    # Clean up
    print("\nShutting down...")
    proc.terminate()
    try:
        proc.wait(timeout=5)
    except subprocess.TimeoutExpired:
        proc.kill()
        proc.wait()
    
    print("Stopping mock server...")
    mock_server_proc.terminate()
    try:
        mock_server_proc.wait(timeout=5)
    except subprocess.TimeoutExpired:
        mock_server_proc.kill()
        mock_server_proc.wait()
    
    # Return success if all tests passed
    return 0 if failure_count == 0 else 1


if __name__ == "__main__":
    sys.exit(main())