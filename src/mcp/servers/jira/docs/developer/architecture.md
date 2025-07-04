# Architecture Overview

This document explains the architecture of the Jira MCP Server, providing developers with an understanding of how the different components work together.

## High-Level Architecture

The Jira MCP Server is built with a modular architecture consisting of the following main components:

```
┌────────────────┐     ┌────────────────┐     ┌────────────────┐
│                │     │                │     │                │
│  MCP Interface │     │  Core Services │     │   Jira API     │
│  (Standard &   │◄────┤  & Tools       │◄────┤   Client       │
│   FastMCP)     │     │                │     │                │
│                │     │                │     │                │
└────────────────┘     └────────────────┘     └────────────────┘
```

### Key Components

1. **MCP Interface Layer**
   - Standard MCP interface (stdin/stdout)
   - FastMCP/FastAPI web interface (HTTP endpoints)

2. **Core Services & Tools**
   - Command processing
   - Tool implementation
   - Response formatting

3. **Jira API Client Layer**
   - Authentication
   - Data retrieval
   - Error handling

## Module Structure

The package is organized into several modules, each with a specific responsibility:

```
src/jira_mcp/
├── __init__.py          # Package initialization
├── __main__.py          # Entry point and CLI handling
├── client.py            # Jira API client implementation
├── formatters.py        # Data formatting utilities
├── tools.py             # MCP tool implementations
├── server.py            # Standard MCP server implementation
└── fastmcp_server.py    # FastMCP server implementation
```

### Entry Point (`__main__.py`)

The entry point handles command-line arguments and dispatches to the appropriate server implementation based on the selected mode (standard or web).

### Standard MCP Server (`server.py`)

This module implements the standard MCP protocol:
- Reads JSON messages from stdin
- Processes commands using the tool implementations
- Writes responses to stdout

### FastMCP Server (`fastmcp_server.py`)

This module implements the FastMCP/FastAPI web interface:
- Defines REST API endpoints using FastAPI
- Integrates with the FastMCP library
- Handles HTTP request/response flow
- Defines Pydantic data models for request/response validation

### Tools Implementation (`tools.py`)

This module contains the implementations of all the MCP tools:
- `mcp_list_assigned_tickets`
- `mcp_list_recent_tickets`
- `mcp_view_ticket`
- `mcp_view_ticket_comments`

### Jira Client (`client.py`)

This module handles interactions with the Jira API:
- Authentication with the Jira API
- Retrieving tickets and comments
- Converting Jira data to a normalized format

### Formatters (`formatters.py`)

This module provides utilities for formatting data:
- Converting tickets to markdown
- Generating markdown tables

## Data Flow

### Standard MCP Mode

```
                                                ┌──────────────────────────┐
                                                │                          │
                                                │      Jira API            │
                                                │                          │
                                                └───────────┬──────────────┘
                                                            │
                                                            │
┌──────────────────────┐    ┌────────────────────┐    ┌─────▼──────────────┐
│                      │    │                    │    │                     │
│  JSON Input          │    │  Command           │    │  JiraTicketManager  │
│  (stdin)        ────────► │  Processing   ────────► │                     │
│                      │    │                    │    │                     │
└──────────────────────┘    └────────────────────┘    └─────┬──────────────┘
                                     │                       │
                                     │                       │
                                     │                       │
                            ┌────────▼───────────┐    ┌──────▼─────────────┐
                            │                    │    │                    │
                            │  Format Response   │◄───┤  Process Data      │
                            │                    │    │                    │
                            └────────┬───────────┘    └────────────────────┘
                                     │
                                     │
                            ┌────────▼───────────┐
                            │                    │
                            │  JSON Output       │
                            │  (stdout)          │
                            │                    │
                            └────────────────────┘
```

### Web Mode (FastAPI/FastMCP)

```
                                                ┌──────────────────────────┐
                                                │                          │
                                                │      Jira API            │
                                                │                          │
                                                └───────────┬──────────────┘
                                                            │
                                                            │
┌──────────────────────┐    ┌────────────────────┐    ┌─────▼──────────────┐
│                      │    │                    │    │                     │
│  HTTP Request        │    │  FastAPI           │    │  JiraTicketManager  │
│                 ────────► │  Endpoint    ─────────► │                     │
│                      │    │  Handler           │    │                     │
└──────────────────────┘    └────────────────────┘    └─────┬──────────────┘
                                     │                       │
                                     │                       │
                                     │                       │
                            ┌────────▼───────────┐    ┌──────▼─────────────┐
                            │                    │    │                    │
                            │  Pydantic Model   │◄───┤  Process Data      │
                            │  Validation        │    │                    │
                            └────────┬───────────┘    └────────────────────┘
                                     │
                                     │
                            ┌────────▼───────────┐
                            │                    │
                            │  HTTP Response     │
                            │  (JSON)            │
                            │                    │
                            └────────────────────┘
```

## Key Design Decisions

### 1. Dual Interface Support

The server supports both standard MCP and FastAPI interfaces, allowing it to be used in different environments. The shared core logic ensures consistent behavior across interfaces.

### 2. Modular Design

The separation of concerns between modules makes the codebase easier to maintain and extend. Each module has a specific responsibility.

### 3. Error Handling Strategy

The server handles errors differently depending on the interface:

- **Standard MCP**: Returns error responses with descriptive messages.
- **FastAPI**: Raises appropriate HTTP exceptions with status codes.

### 4. Markdown Formatting

The server provides both raw data and pre-formatted markdown for each response, making it easy for clients to display the information in different formats.

### 5. Pydantic Models

FastAPI uses Pydantic models for request/response validation, providing automatic type checking and documentation generation.

## Integration Points

### Integration with MCP Framework

The standard MCP server integrates with the MCP framework by:
- Implementing the MCP protocol
- Processing commands in the expected format
- Returning responses in the expected format

### Integration with FastMCP/FastAPI

The web server integrates with FastMCP and FastAPI by:
- Using the `@mcp.tool()` decorator to expose tools as HTTP endpoints
- Defining Pydantic models for request/response validation
- Using FastAPI's dependency injection system

### Integration with Jira API

The server integrates with the Jira API using the `jira` Python package, handling:
- Authentication
- Retrieving tickets and comments
- Error handling

## Next Steps

- [Module Reference](./modules.md) - Detailed documentation of each module
- [Extending the Server](./extending.md) - How to extend the server
- [Testing Guide](./testing.md) - How to test the server