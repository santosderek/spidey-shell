# API Reference

This document describes all the APIs available in the Jira MCP Server, including both standard MCP commands and REST endpoints.

## Standard MCP Commands

These commands are available in the standard MCP interface.

### `list-commands`

Lists all available commands.

**Request:**
```json
{
  "command": "list-commands"
}
```

**Response:**
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
    {
      "name": "jira.view-ticket",
      "description": "View details of a specific ticket",
      "args": {
        "ticket_id": {
          "type": "string",
          "required": true,
          "description": "Jira ticket ID"
        }
      }
    },
    {
      "name": "jira.view-ticket-comments",
      "description": "View comments on a specific ticket",
      "args": {
        "ticket_id": {
          "type": "string",
          "required": true,
          "description": "Jira ticket ID"
        }
      }
    }
  ]
}
```

### `jira.list-assigned-tickets`

Lists tickets assigned to the current user.

**Request:**
```json
{
  "command": "jira.list-assigned-tickets"
}
```

**Response:**
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

**Error Response:**
```json
{
  "message": "No assigned tickets found",
  "tickets": [],
  "count": 0,
  "markdown": "No tickets found."
}
```

### `jira.list-recent-tickets`

Lists tickets updated within a specified number of days.

**Request:**
```json
{
  "command": "jira.list-recent-tickets",
  "args": {
    "days": 14
  }
}
```

**Response:**
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

**Error Response:**
```json
{
  "message": "No tickets updated in the last 14 days",
  "tickets": [],
  "count": 0,
  "markdown": "No tickets found."
}
```

### `jira.view-ticket`

Shows details of a specific ticket.

**Request:**
```json
{
  "command": "jira.view-ticket",
  "args": {
    "ticket_id": "PROJECT-123"
  }
}
```

**Response:**
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

**Error Response:**
A `ValueError` is raised with these messages:
- `"No ticket ID provided"` - When no ticket ID is provided
- `"Ticket not found: {ticket_id}"` - When the ticket is not found

### `jira.view-ticket-comments`

Shows comments on a specific ticket.

**Request:**
```json
{
  "command": "jira.view-ticket-comments",
  "args": {
    "ticket_id": "PROJECT-123"
  }
}
```

**Response:**
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

**Error Response:**
A `ValueError` is raised with this message:
- `"No ticket ID provided"` - When no ticket ID is provided

On empty comments:
```json
{
  "message": "No comments found for ticket: PROJECT-123",
  "ticket_id": "PROJECT-123",
  "comments": [],
  "count": 0,
  "markdown": "No comments found."
}
```

## REST API Endpoints

These endpoints are available in web server mode.

### `GET /`

Returns information about the server.

**Response:**
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

### `GET /tickets/assigned`

Returns tickets assigned to the current user.

**Response:**
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

**Error Response:**
```json
{
  "message": "No assigned tickets found",
  "tickets": [],
  "count": 0,
  "markdown": "No tickets found."
}
```

### `GET /tickets/recent`

Returns tickets updated within the specified number of days.

**Parameters:**
- `days` (query, optional): Number of days to look back (default: 30)

**Response:**
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

**Error Response:**
```json
{
  "message": "No tickets updated in the last 30 days",
  "tickets": [],
  "count": 0,
  "markdown": "No tickets found."
}
```

### `GET /tickets/{ticket_id}`

Returns details of the specified ticket.

**Parameters:**
- `ticket_id` (path, required): ID of the ticket to view

**Response:**
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

**Error Response:**
```json
{
  "detail": "Ticket not found: PROJECT-123"
}
```

### `GET /tickets/{ticket_id}/comments`

Returns comments on the specified ticket.

**Parameters:**
- `ticket_id` (path, required): ID of the ticket to view comments for

**Response:**
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

**Error Response:**
```json
{
  "detail": "No ticket ID provided"
}
```

On empty comments:
```json
{
  "message": "No comments found for ticket: PROJECT-123",
  "ticket_id": "PROJECT-123",
  "comments": [],
  "count": 0,
  "markdown": "No comments found."
}
```

## Data Models

### Ticket

A Jira ticket with the following properties:

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Ticket ID (e.g., "PROJECT-123") |
| `title` | string | Ticket title/summary |
| `status` | string | Current status (e.g., "Open", "In Progress") |
| `url` | string | URL to the ticket in Jira |
| `priority` | string, optional | Ticket priority (e.g., "High", "Medium") |
| `assigned_to` | string, optional | Name of the assignee |
| `description` | string, optional | Ticket description |
| `created_at` | string, optional | Creation timestamp |
| `updated_at` | string, optional | Last update timestamp |
| `last_comment_at` | string, optional | Last comment timestamp |

### ListTicketsResponse

Response model for listing tickets:

| Field | Type | Description |
|-------|------|-------------|
| `tickets` | array of Ticket | List of tickets |
| `count` | integer | Number of tickets |
| `markdown` | string | Formatted markdown table of tickets |
| `message` | string, optional | Additional message (e.g., "No tickets found") |

### ViewTicketResponse

Response model for viewing a ticket:

| Field | Type | Description |
|-------|------|-------------|
| `ticket` | Ticket | The ticket details |
| `markdown` | string | Formatted markdown representation of the ticket |

### ViewCommentsResponse

Response model for viewing ticket comments:

| Field | Type | Description |
|-------|------|-------------|
| `ticket_id` | string | ID of the ticket |
| `comments` | array of string | List of comments |
| `count` | integer | Number of comments |
| `markdown` | string | Formatted markdown representation of the comments |
| `message` | string, optional | Additional message (e.g., "No comments found") |

### ErrorResponse

Response model for errors:

| Field | Type | Description |
|-------|------|-------------|
| `error` | string | Error message |
| `available_commands` | array of string, optional | List of available commands (for unknown command errors) |