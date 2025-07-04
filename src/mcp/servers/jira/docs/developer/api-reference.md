# Developer API Reference

This document provides a detailed API reference for developers working with the Jira MCP Server codebase.

## Module: `client.py`

### `JiraTicketManager`

Core class for interacting with the Jira API.

#### Constructor

```python
def __init__(self)
```

Initializes the manager with credentials from environment variables.

**Environment Variables:**
- `JIRA_URL`: URL of the Jira instance
- `JIRA_API_USER`: Jira username
- `JIRA_API_TOKEN`: Jira API token

**Default Values:**
- Cisco Jira defaults if no URL is provided:
  - `jira_cloud_scheme`: "https"
  - `jira_cloud_domain`: "cisco-jira.atlassian.net"

#### Methods

```python
def get_assigned_tickets(self) -> List[Dict[str, Any]]
```

Gets tickets assigned to the current user.

**Returns:** List of ticket dictionaries.

```python
def get_recent_tickets(self, days: int) -> List[Dict[str, Any]]
```

Gets tickets updated within the specified number of days.

**Parameters:**
- `days`: Number of days to look back.

**Returns:** List of ticket dictionaries.

```python
def get_ticket_by_id(self, ticket_id: str) -> Optional[Dict[str, Any]]
```

Gets details of a specific ticket.

**Parameters:**
- `ticket_id`: ID of the ticket (e.g., "PROJECT-123").

**Returns:** Ticket dictionary or None if not found.

```python
def get_ticket_comments(self, ticket_id: str) -> List[str]
```

Gets comments for a specific ticket.

**Parameters:**
- `ticket_id`: ID of the ticket.

**Returns:** List of comment strings formatted as markdown.

```python
def _convert_to_ticket(self, issue) -> Dict[str, Any]
```

*(Private)* Converts a Jira issue to a normalized ticket format.

**Parameters:**
- `issue`: Jira issue object.

**Returns:** Normalized ticket dictionary.

## Module: `formatters.py`

### Functions

```python
def format_date(date_str: str, reference_date: Optional[datetime] = None) -> str
```

Formats a date string for display, with relative formatting for recent dates.

**Parameters:**
- `date_str`: ISO format date string from Jira.
- `reference_date`: Optional reference date for relative formatting. Defaults to the current time.

**Returns:** Formatted date string.

```python
def generate_markdown_table(tickets: List[Dict[str, Any]]) -> str
```

Generates a markdown table from a list of tickets.

**Parameters:**
- `tickets`: List of ticket dictionaries.

**Returns:** Markdown table string.

```python
def ticket_to_markdown(ticket: Dict[str, Any]) -> str
```

Converts a ticket to a detailed markdown representation.

**Parameters:**
- `ticket`: Ticket dictionary.

**Returns:** Markdown string.

## Module: `tools.py`

### Functions

```python
def mcp_list_assigned_tickets(args: Dict[str, Any]) -> Dict[str, Any]
```

MCP tool to list tickets assigned to the current user.

**Parameters:**
- `args`: Dictionary of arguments (not used).

**Returns:** Dictionary with:
- `tickets`: List of ticket dictionaries.
- `count`: Number of tickets.
- `markdown`: Markdown table representation.
- `message`: Optional status message.

```python
def mcp_list_recent_tickets(args: Dict[str, Any]) -> Dict[str, Any]
```

MCP tool to list recently updated tickets.

**Parameters:**
- `args`: Dictionary with:
  - `days`: Number of days to look back (optional, default 30).

**Returns:** Dictionary with:
- `tickets`: List of ticket dictionaries.
- `count`: Number of tickets.
- `markdown`: Markdown table representation.
- `message`: Optional status message.

```python
def mcp_view_ticket(args: Dict[str, Any]) -> Dict[str, Any]
```

MCP tool to view details of a specific ticket.

**Parameters:**
- `args`: Dictionary with:
  - `ticket_id`: ID of the ticket to view (required).

**Returns:** Dictionary with:
- `ticket`: Ticket dictionary.
- `markdown`: Markdown representation.

**Raises:**
- `ValueError`: If no ticket ID is provided or ticket is not found.

```python
def mcp_view_ticket_comments(args: Dict[str, Any]) -> Dict[str, Any]
```

MCP tool to view comments on a specific ticket.

**Parameters:**
- `args`: Dictionary with:
  - `ticket_id`: ID of the ticket to view comments for (required).

**Returns:** Dictionary with:
- `ticket_id`: ID of the ticket.
- `comments`: List of comment strings.
- `count`: Number of comments.
- `markdown`: Markdown representation.
- `message`: Optional status message.

**Raises:**
- `ValueError`: If no ticket ID is provided.

### Constants

```python
MCP_TOOLS: Dict[str, Dict[str, Any]]
```

Dictionary mapping command names to their implementations.

**Structure:**
```python
{
    "jira.list-assigned-tickets": {
        "function": mcp_list_assigned_tickets,
        "description": "List tickets assigned to the current user",
        "args": {}
    },
    "jira.list-recent-tickets": {
        "function": mcp_list_recent_tickets,
        "description": "List recently updated tickets",
        "args": {
            "days": {
                "type": "integer",
                "default": 30,
                "description": "Number of days to look back"
            }
        }
    },
    # ...
}
```

## Module: `server.py`

### Functions

```python
def configure_logging(debug: bool = False) -> None
```

Configures logging for the MCP server.

**Parameters:**
- `debug`: Boolean indicating whether to enable debug logging.

```python
def process_message(message: Dict[str, Any]) -> Dict[str, Any]
```

Processes an incoming MCP message and returns a response.

**Parameters:**
- `message`: Dictionary with:
  - `command`: Command name.
  - `args`: Command arguments (optional).

**Returns:** Response dictionary.

```python
def main() -> int
```

Runs the Jira MCP server in standard MCP mode.

**Returns:** Exit code.

## Module: `fastmcp_server.py`

### Constants

```python
app: FastAPI
```

FastAPI application instance.

```python
mcp: FastMCP
```

FastMCP server instance.

### Pydantic Models

```python
class ListTicketsRequest(BaseModel)
```

Pydantic model for list tickets request.

**Fields:**
- `filter`: Literal["assigned", "recent"] - Filter type.
- `days`: int - Number of days to look back.

```python
class ViewTicketRequest(BaseModel)
```

Pydantic model for view ticket request.

**Fields:**
- `ticket_id`: str - ID of the ticket to view.

```python
class Ticket(BaseModel)
```

Pydantic model for a ticket.

**Fields:**
- `id`: str - Ticket ID.
- `title`: str - Ticket title.
- `status`: str - Ticket status.
- `url`: str - URL to the ticket.
- `priority`: Optional[str] - Ticket priority.
- `assigned_to`: Optional[str] - Assigned user.
- `description`: Optional[str] - Ticket description.
- `created_at`: Optional[str] - Creation date.
- `updated_at`: Optional[str] - Update date.
- `last_comment_at`: Optional[str] - Last comment date.

```python
class ListTicketsResponse(BaseModel)
```

Pydantic model for list tickets response.

**Fields:**
- `tickets`: List[Ticket] - List of tickets.
- `count`: int - Number of tickets.
- `markdown`: str - Markdown representation.
- `message`: Optional[str] - Status message.

```python
class ViewTicketResponse(BaseModel)
```

Pydantic model for view ticket response.

**Fields:**
- `ticket`: Ticket - Ticket details.
- `markdown`: str - Markdown representation.

```python
class ViewCommentsResponse(BaseModel)
```

Pydantic model for view comments response.

**Fields:**
- `ticket_id`: str - ID of the ticket.
- `comments`: List[str] - List of comments.
- `count`: int - Number of comments.
- `markdown`: str - Markdown representation.
- `message`: Optional[str] - Status message.

```python
class ErrorResponse(BaseModel)
```

Pydantic model for error response.

**Fields:**
- `error`: str - Error message.

### FastMCP Tool Functions

```python
@mcp.tool()
def list_assigned_tickets(args: Optional[Dict[str, Any]] = None) -> Dict[str, Any]
```

Lists tickets assigned to the current user.

**Parameters:**
- `args`: Optional dictionary of arguments.

**Returns:** Dictionary with tickets, count, and markdown.

```python
@mcp.tool()
def list_recent_tickets(args: Dict[str, Any]) -> Dict[str, Any]
```

Lists recently updated tickets.

**Parameters:**
- `args`: Dictionary with optional `days` parameter.

**Returns:** Dictionary with tickets, count, and markdown.

```python
@mcp.tool()
def view_ticket(args: Dict[str, Any]) -> Dict[str, Any]
```

Views details of a specific ticket.

**Parameters:**
- `args`: Dictionary with `ticket_id` parameter.

**Returns:** Dictionary with ticket and markdown.

**Raises:**
- `ValueError`: If no ticket ID is provided or ticket is not found.

```python
@mcp.tool()
def view_ticket_comments(args: Dict[str, Any]) -> Dict[str, Any]
```

Views comments on a specific ticket.

**Parameters:**
- `args`: Dictionary with `ticket_id` parameter.

**Returns:** Dictionary with comments, count, and markdown.

**Raises:**
- `ValueError`: If no ticket ID is provided.

### FastAPI Endpoints

```python
@app.get("/")
async def root()
```

Root endpoint providing server information.

**Returns:** Server information dictionary.

```python
@app.get("/tickets/assigned", response_model=ListTicketsResponse)
async def get_assigned_tickets()
```

Gets tickets assigned to the current user.

**Returns:** ListTicketsResponse object.

**Raises:**
- `HTTPException`: If an error occurs.

```python
@app.get("/tickets/recent", response_model=ListTicketsResponse)
async def get_recent_tickets(days: int = 30)
```

Gets tickets updated within the specified number of days.

**Parameters:**
- `days`: Number of days to look back (default: 30).

**Returns:** ListTicketsResponse object.

**Raises:**
- `HTTPException`: If an error occurs.

```python
@app.get("/tickets/{ticket_id}", response_model=ViewTicketResponse)
async def get_ticket(ticket_id: str)
```

Gets details of a specific ticket.

**Parameters:**
- `ticket_id`: ID of the ticket to view.

**Returns:** ViewTicketResponse object.

**Raises:**
- `HTTPException`: If ticket not found (404) or other error occurs (500).

```python
@app.get("/tickets/{ticket_id}/comments", response_model=ViewCommentsResponse)
async def get_ticket_comments(ticket_id: str)
```

Gets comments on a specific ticket.

**Parameters:**
- `ticket_id`: ID of the ticket to view comments for.

**Returns:** ViewCommentsResponse object.

**Raises:**
- `HTTPException`: If ticket not found (404) or other error occurs (500).

### Server Functions

```python
def run_server(host="0.0.0.0", port=8000, log_level="info")
```

Runs the FastAPI server.

**Parameters:**
- `host`: Host to bind to (default: "0.0.0.0").
- `port`: Port to listen on (default: 8000).
- `log_level`: Logging level (default: "info").

## Module: `__main__.py`

### Functions

```python
def main()
```

Parses arguments and runs in the appropriate mode.

**Command-line Arguments:**
- `--mode`: Server mode (standard or web).
- `--port`: Port to use for web server mode.
- `--host`: Host to bind to in web server mode.
- `--debug`: Enable debug logging.

## Data Structures

### Ticket Dictionary

```python
{
    "id": str,              # Ticket ID (e.g., "PROJECT-123")
    "title": str,           # Ticket title/summary
    "status": str,          # Current status (e.g., "Open", "In Progress")
    "url": str,             # URL to the ticket in Jira
    "priority": str,        # Optional: Ticket priority (e.g., "High")
    "assigned_to": str,     # Optional: Name of the assignee
    "description": str,     # Optional: Ticket description
    "created_at": str,      # Optional: Creation timestamp
    "updated_at": str,      # Optional: Last update timestamp
    "last_comment_at": str, # Optional: Last comment timestamp
}
```

### MCP Message Dictionary

```python
{
    "command": str,         # Command name (e.g., "jira.list-assigned-tickets")
    "args": dict,           # Optional: Command arguments
}
```

### MCP Response Dictionary

For successful responses:

```python
{
    "tickets": [ticket_dict, ...],  # List of ticket dictionaries
    "count": int,                   # Number of tickets
    "markdown": str,                # Markdown representation
    "message": str,                 # Optional: Status message
}
```

Or:

```python
{
    "ticket": ticket_dict,          # Ticket dictionary
    "markdown": str,                # Markdown representation
}
```

Or:

```python
{
    "ticket_id": str,               # ID of the ticket
    "comments": [str, ...],         # List of comment strings
    "count": int,                   # Number of comments
    "markdown": str,                # Markdown representation
    "message": str,                 # Optional: Status message
}
```

For error responses:

```python
{
    "error": str,                   # Error message
    "available_commands": [str, ...], # Optional: Available commands
}
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `JIRA_URL` | URL of the Jira instance | For Cisco: "https://cisco-jira.atlassian.net" |
| `JIRA_API_USER` | Jira username | None |
| `JIRA_API_TOKEN` | Jira API token | None |

## Command-line Arguments

| Argument | Description | Default |
|----------|-------------|---------|
| `--mode` | Server mode (standard or web) | "standard" |
| `--port` | Port for web server mode | 8000 |
| `--host` | Host for web server mode | "0.0.0.0" |
| `--debug` | Enable debug logging | False |

## Next Steps

Now that you understand the API, consider:
- [Extending the Server](./extending.md)
- [Testing Guide](./testing.md)
- [Contributing Guidelines](./contributing.md)