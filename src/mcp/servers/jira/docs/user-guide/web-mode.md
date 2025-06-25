# Web Server Mode

The Web Server mode of the Jira MCP Server exposes HTTP endpoints using FastAPI, allowing you to integrate with the server using standard web technologies.

## Starting the Server in Web Mode

```bash
# Run in web mode with default options (port 8000, host 0.0.0.0)
python -m jira_mcp --mode web

# Specify a custom port and host
python -m jira_mcp --mode web --port 9000 --host 127.0.0.1

# Enable debug logging
python -m jira_mcp --mode web --debug
```

## Available Endpoints

Once the server is running in web mode, the following endpoints are available:

### Root Endpoint

```
GET /
```

Returns basic information about the server:

```json
{
  "name": "Jira MCP Server",
  "version": "0.1.0",
  "description": "MCP Server for Jira integration",
  "endpoints": [
    "/tickets/assigned",
    "/tickets/recent",
    "/tickets/{ticket_id}",
    "/tickets/{ticket_id}/comments"
  ]
}
```

### List Assigned Tickets

```
GET /tickets/assigned
```

Returns tickets assigned to the current user:

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

### List Recent Tickets

```
GET /tickets/recent?days=14
```

Returns tickets updated within the specified number of days (default: 30):

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

### View Ticket

```
GET /tickets/{ticket_id}
```

Returns details of the specified ticket:

```json
{
  "ticket": {
    "id": "PROJECT-123",
    "title": "Fix the login bug",
    "status": "In Progress",
    "url": "https://your-domain.atlassian.net/browse/PROJECT-123",
    "priority": "High",
    "assigned_to": "John Doe",
    "description": "The login button doesn't work when the username contains special characters.",
    "created_at": "2023-06-01T09:00:00.000+0000",
    "updated_at": "2023-06-10T15:30:00.000+0000"
  },
  "markdown": "# PROJECT-123: Fix the login bug\n\n**Status:** In Progress  \n**Priority:** High  \n**Assigned to:** John Doe  \n\n## Description\nThe login button doesn't work when the username contains special characters.\n\n**Created:** 2023-06-01  \n**Updated:** 2023-06-10"
}
```

### View Ticket Comments

```
GET /tickets/{ticket_id}/comments
```

Returns comments on the specified ticket:

```json
{
  "ticket_id": "PROJECT-123",
  "comments": [
    "**Jane Smith** on 2023-06-05 10:15:20:\n\nI've reproduced this issue with usernames containing @ and % characters.\n",
    "**John Doe** on 2023-06-10 15:30:00:\n\nWorking on a fix, should be done by tomorrow.\n"
  ],
  "count": 2,
  "markdown": "**Jane Smith** on 2023-06-05 10:15:20:\n\nI've reproduced this issue with usernames containing @ and % characters.\n\n---\n**John Doe** on 2023-06-10 15:30:00:\n\nWorking on a fix, should be done by tomorrow.\n"
}
```

## Error Handling

When an error occurs, the server returns an appropriate HTTP status code and a JSON response:

### 404 Not Found

```json
{
  "detail": "Ticket not found: INVALID-123"
}
```

### 500 Internal Server Error

```json
{
  "detail": "Error connecting to Jira API"
}
```

## Examples

### Using with cURL

```bash
# Get server information
curl http://localhost:8000/

# List assigned tickets
curl http://localhost:8000/tickets/assigned

# List recent tickets (last 7 days)
curl http://localhost:8000/tickets/recent?days=7

# View a specific ticket
curl http://localhost:8000/tickets/PROJECT-123

# View comments on a specific ticket
curl http://localhost:8000/tickets/PROJECT-123/comments
```

### Using with JavaScript/TypeScript

```typescript
// Using fetch API
async function getAssignedTickets() {
  const response = await fetch('http://localhost:8000/tickets/assigned');
  const data = await response.json();
  
  console.log(`Found ${data.count} assigned tickets`);
  console.log(data.markdown);
  
  return data.tickets;
}

// Using axios
import axios from 'axios';

async function getTicketDetails(ticketId) {
  try {
    const response = await axios.get(`http://localhost:8000/tickets/${ticketId}`);
    return response.data;
  } catch (error) {
    if (error.response && error.response.status === 404) {
      console.error(`Ticket ${ticketId} not found`);
    } else {
      console.error(`Error fetching ticket ${ticketId}:`, error);
    }
    return null;
  }
}
```

### Using with Python

```python
import requests

# Base URL of the Jira MCP server
BASE_URL = "http://localhost:8000"

def get_assigned_tickets():
    """Get tickets assigned to the current user."""
    response = requests.get(f"{BASE_URL}/tickets/assigned")
    response.raise_for_status()  # Raise exception for 4XX/5XX responses
    data = response.json()
    return data["tickets"], data["markdown"]

def get_ticket(ticket_id):
    """Get details of a specific ticket."""
    try:
        response = requests.get(f"{BASE_URL}/tickets/{ticket_id}")
        response.raise_for_status()
        return response.json()
    except requests.exceptions.HTTPError as err:
        if err.response.status_code == 404:
            print(f"Ticket {ticket_id} not found")
        else:
            print(f"Error fetching ticket: {err}")
        return None
```

## Swagger Documentation

FastAPI automatically generates interactive API documentation for the server. You can access it at:

```
http://localhost:8000/docs
```

This provides a web interface where you can:
- See all available endpoints
- Understand the request parameters and response models
- Try out the API directly in your browser

## Cross-Origin Resource Sharing (CORS)

The server is configured with CORS middleware that allows requests from any origin (`*`). For production use, you may want to restrict this to specific origins.

## Next Steps

- [API Reference](./api-reference.md) - Complete API reference
- [Troubleshooting](./troubleshooting.md) - Common issues and solutions