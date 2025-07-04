# Standard MCP Mode

The Standard MCP (Model Calling Protocol) mode allows Jira MCP Server to communicate via standard input and output, following the MCP protocol. This mode is useful for integration with CLI tools and other MCP clients.

## Starting the Server in Standard Mode

```bash
# Default mode is standard
python -m jira_mcp

# Explicitly specify standard mode
python -m jira_mcp --mode standard

# Enable debug logging
python -m jira_mcp --debug
```

## MCP Protocol Overview

The Model Calling Protocol is a simple JSON-based protocol for communication between a client and server. Each message is a JSON object sent on a single line.

### Request Format

A typical MCP request looks like this:

```json
{
  "command": "jira.list-assigned-tickets",
  "args": {}
}
```

The request contains:
- `command`: The name of the command to execute
- `args`: Optional arguments for the command

### Response Format

A typical MCP response looks like this:

```json
{
  "tickets": [
    {
      "id": "PROJECT-123",
      "title": "Fix the login bug",
      "status": "In Progress",
      "url": "https://your-domain.atlassian.net/browse/PROJECT-123",
      "priority": "High",
      "assigned_to": "John Doe",
      "created_at": "2023-06-01T09:00:00.000+0000",
      "updated_at": "2023-06-10T15:30:00.000+0000"
    }
  ],
  "count": 1,
  "markdown": "| ID | Title | Status | Priority |\n|---|---|---|---|\n| [PROJECT-123](https://your-domain.atlassian.net/browse/PROJECT-123) | Fix the login bug | In Progress | High |"
}
```

Error responses include an `error` field:

```json
{
  "error": "Unknown command: unknown-command",
  "available_commands": ["jira.list-assigned-tickets", "jira.list-recent-tickets", "jira.view-ticket", "jira.view-ticket-comments", "list-commands"]
}
```

## Available Commands

### List Commands

Lists all available commands:

```json
{
  "command": "list-commands"
}
```

Response:

```json
{
  "commands": [
    {
      "name": "jira.list-assigned-tickets",
      "description": "List tickets assigned to the current user",
      "args": {}
    },
    {
      "name": "jira.list-recent-tickets",
      "description": "List recently updated tickets",
      "args": {
        "days": {
          "type": "integer",
          "default": 30,
          "description": "Number of days to look back"
        }
      }
    },
    ...
  ]
}
```

### List Assigned Tickets

Lists tickets assigned to the current user:

```json
{
  "command": "jira.list-assigned-tickets"
}
```

### List Recent Tickets

Lists tickets updated within a specified number of days:

```json
{
  "command": "jira.list-recent-tickets",
  "args": {
    "days": 14
  }
}
```

### View Ticket

Shows details of a specific ticket:

```json
{
  "command": "jira.view-ticket",
  "args": {
    "ticket_id": "PROJECT-123"
  }
}
```

### View Ticket Comments

Shows comments on a specific ticket:

```json
{
  "command": "jira.view-ticket-comments",
  "args": {
    "ticket_id": "PROJECT-123"
  }
}
```

## Examples

### Using in Bash

You can use the server with standard Unix tools:

```bash
# List assigned tickets
echo '{"command": "jira.list-assigned-tickets"}' | python -m jira_mcp

# List recent tickets (last 7 days)
echo '{"command": "jira.list-recent-tickets", "args": {"days": 7}}' | python -m jira_mcp

# View a specific ticket
echo '{"command": "jira.view-ticket", "args": {"ticket_id": "PROJECT-123"}}' | python -m jira_mcp

# List all available commands
echo '{"command": "list-commands"}' | python -m jira_mcp
```

### Usage in Python

```python
import json
import subprocess

def run_mcp_command(command, args=None):
    """Run a command with the Jira MCP server."""
    if args is None:
        args = {}
    
    request = json.dumps({"command": command, "args": args})
    process = subprocess.Popen(
        ["python", "-m", "jira_mcp"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        text=True
    )
    stdout, _ = process.communicate(request)
    return json.loads(stdout)

# Example usage
tickets = run_mcp_command("jira.list-assigned-tickets")
print(f"Found {tickets['count']} assigned tickets")

# View a specific ticket
ticket_details = run_mcp_command("jira.view-ticket", {"ticket_id": "PROJECT-123"})
print(ticket_details["markdown"])
```

## Error Handling

When an error occurs, the server returns a JSON object with an `error` field:

```json
{
  "error": "No ticket ID provided"
}
```

Common errors:

- Invalid command: `Unknown command: [command]`
- Missing arguments: `No ticket ID provided`
- Jira API errors: `Error fetching ticket: [error message]`
- Server errors: `Internal server error: [error message]`

## Next Steps

- [Web Server Mode](./web-mode.md) - Using the FastAPI web interface
- [API Reference](./api-reference.md) - Complete API reference
- [Troubleshooting](./troubleshooting.md) - Common issues and solutions