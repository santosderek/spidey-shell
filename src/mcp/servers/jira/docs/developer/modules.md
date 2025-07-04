# Module Reference

This document provides detailed information about each module in the Jira MCP Server.

## `__main__.py`

The entry point for the package when run as a module.

### Key Functions

#### `main()`
- **Description**: Parses command-line arguments and runs the server in the appropriate mode.
- **Arguments**: None
- **Returns**: Integer exit code

### Command-line Arguments

- `--mode`: Server mode (`standard` or `web`)
- `--port`: Port to use for web server mode
- `--host`: Host to bind to in web server mode
- `--debug`: Enable debug logging

### Dependencies

- `argparse` for command-line argument parsing
- `logging` for logging configuration
- `jira_mcp.server.main` for standard mode
- `jira_mcp.fastmcp_server.run_server` for web mode

## `client.py`

Provides the `JiraTicketManager` class for interacting with the Jira API.

### Classes

#### `JiraTicketManager`
- **Description**: Manages interactions with Jira API to fetch and process tickets.
- **Methods**:
  - `__init__()`: Initializes the manager with credentials from environment variables.
  - `get_assigned_tickets()`: Gets tickets assigned to the current user.
  - `get_recent_tickets(days)`: Gets tickets updated within the specified number of days.
  - `get_ticket_by_id(ticket_id)`: Gets details of a specific ticket.
  - `get_ticket_comments(ticket_id)`: Gets comments for a specific ticket.
  - `_convert_to_ticket(issue)`: Converts a Jira issue to a normalized ticket format.

### Environment Variables

- `JIRA_URL`: URL of the Jira instance
- `JIRA_API_USER`: Jira username
- `JIRA_API_TOKEN`: Jira API token

### Dependencies

- `jira` package for Jira API interactions
- `datetime` for date handling
- `os` for environment variable access
- `logging` for logging

## `formatters.py`

Provides utility functions for formatting data.

### Functions

#### `format_date(date_str, reference_date=None)`
- **Description**: Formats a date string for display.
- **Arguments**:
  - `date_str`: ISO format date string from Jira.
  - `reference_date`: Optional reference date for relative formatting.
- **Returns**: Formatted date string.

#### `generate_markdown_table(tickets)`
- **Description**: Generates a markdown table from a list of tickets.
- **Arguments**:
  - `tickets`: List of ticket dictionaries.
- **Returns**: Markdown table string.

#### `ticket_to_markdown(ticket)`
- **Description**: Converts a ticket to a detailed markdown representation.
- **Arguments**:
  - `ticket`: Ticket dictionary.
- **Returns**: Markdown string.

### Dependencies

- `datetime` for date parsing and formatting

## `tools.py`

Implements the MCP tools for Jira integration.

### Functions

#### `mcp_list_assigned_tickets(args)`
- **Description**: Lists tickets assigned to the current user.
- **Arguments**:
  - `args`: Dictionary of arguments (not used).
- **Returns**: Dictionary with tickets, count, and markdown.

#### `mcp_list_recent_tickets(args)`
- **Description**: Lists tickets updated within a specified number of days.
- **Arguments**:
  - `args`: Dictionary with optional `days` parameter.
- **Returns**: Dictionary with tickets, count, and markdown.

#### `mcp_view_ticket(args)`
- **Description**: Shows details of a specific ticket.
- **Arguments**:
  - `args`: Dictionary with `ticket_id` parameter.
- **Returns**: Dictionary with ticket and markdown.
- **Raises**: `ValueError` if ticket ID is missing or ticket not found.

#### `mcp_view_ticket_comments(args)`
- **Description**: Shows comments on a specific ticket.
- **Arguments**:
  - `args`: Dictionary with `ticket_id` parameter.
- **Returns**: Dictionary with comments, count, and markdown.
- **Raises**: `ValueError` if ticket ID is missing.

### Constants

#### `MCP_TOOLS`
- **Description**: Dictionary mapping command names to their implementations.
- **Keys**: Command names (strings).
- **Values**: Dictionaries with function, description, and args.

### Dependencies

- `jira_mcp.client.JiraTicketManager` for Jira API interactions
- `jira_mcp.formatters` for data formatting

## `server.py`

Implements the standard MCP server that communicates via stdin/stdout.

### Functions

#### `configure_logging(debug=False)`
- **Description**: Configures logging for the MCP server.
- **Arguments**:
  - `debug`: Boolean indicating whether to enable debug logging.
- **Returns**: None

#### `process_message(message)`
- **Description**: Processes an incoming MCP message.
- **Arguments**:
  - `message`: Dictionary with command and args.
- **Returns**: Response dictionary.

#### `main()`
- **Description**: Runs the MCP server.
- **Arguments**: None
- **Returns**: Integer exit code

### Dependencies

- `json` for parsing messages
- `sys` for stdin/stdout access
- `argparse` for command-line argument parsing
- `logging` for logging
- `jira_mcp.tools.MCP_TOOLS` for tool implementations

## `fastmcp_server.py`

Implements the FastMCP server with FastAPI integration.

### Constants

#### `app`
- **Description**: FastAPI application instance.

#### `mcp`
- **Description**: FastMCP server instance.

### Models

#### `ListTicketsRequest`
- **Description**: Pydantic model for list tickets request.
- **Fields**:
  - `filter`: Literal["assigned", "recent"]
  - `days`: int

#### `ViewTicketRequest`
- **Description**: Pydantic model for view ticket request.
- **Fields**:
  - `ticket_id`: str

#### `Ticket`
- **Description**: Pydantic model for a ticket.
- **Fields**: Various ticket properties.

#### `ListTicketsResponse`
- **Description**: Pydantic model for list tickets response.
- **Fields**:
  - `tickets`: List[Ticket]
  - `count`: int
  - `markdown`: str
  - `message`: Optional[str]

#### `ViewTicketResponse`
- **Description**: Pydantic model for view ticket response.
- **Fields**:
  - `ticket`: Ticket
  - `markdown`: str

#### `ViewCommentsResponse`
- **Description**: Pydantic model for view comments response.
- **Fields**:
  - `ticket_id`: str
  - `comments`: List[str]
  - `count`: int
  - `markdown`: str
  - `message`: Optional[str]

#### `ErrorResponse`
- **Description**: Pydantic model for error response.
- **Fields**:
  - `error`: str

### Functions

#### MCP Tool Functions

- `list_assigned_tickets(args)`: Lists assigned tickets.
- `list_recent_tickets(args)`: Lists recent tickets.
- `view_ticket(args)`: Views a specific ticket.
- `view_ticket_comments(args)`: Views comments on a specific ticket.

#### FastAPI Endpoints

- `root()`: Root endpoint returning server information.
- `get_assigned_tickets()`: Endpoint for assigned tickets.
- `get_recent_tickets(days=30)`: Endpoint for recent tickets.
- `get_ticket(ticket_id)`: Endpoint for viewing a ticket.
- `get_ticket_comments(ticket_id)`: Endpoint for viewing ticket comments.

#### Server Functions

- `run_server(host="0.0.0.0", port=8000, log_level="info")`: Runs the FastAPI server.

### Dependencies

- `fastapi` for REST API
- `fastapi.middleware.cors` for CORS support
- `mcp.server.fastmcp` for FastMCP integration
- `pydantic` for data validation
- `jira_mcp.client` for Jira API interactions
- `jira_mcp.formatters` for data formatting
- `uvicorn` for ASGI server

## Interaction Flow

The following diagram illustrates how the modules interact:

```
┌────────────────────┐
│                    │
│    __main__.py     │
│                    │
└───┬────────────┬───┘
    │            │
    │            │
┌───▼────┐   ┌───▼────────┐
│        │   │            │
│server.py│   │fastmcp_server.py│
│        │   │            │
└───┬────┘   └─────┬──────┘
    │              │
    │              │
┌───▼──────────────▼───┐
│                      │
│      tools.py        │
│                      │
└──────────┬───────────┘
           │
           │
┌──────────▼───────────┐
│                      │
│      client.py       │
│                      │
└──────────┬───────────┘
           │
           │
┌──────────▼───────────┐
│                      │
│    formatters.py     │
│                      │
└──────────────────────┘
```

## Next Steps

- [Extending the Server](./extending.md) - How to extend the server
- [Testing Guide](./testing.md) - How to test the server
- [API Reference](./api-reference.md) - Detailed API reference