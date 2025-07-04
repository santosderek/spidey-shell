# Jira MCP Server

A Model Calling Protocol (MCP) server implementation for interacting with Jira tickets.

## Features

- Fetch all Jira tickets assigned to the current user
- Display ticket details in a normalized format
- Generate markdown tables of tickets for UI display
- Get comments for specific tickets

## Installation

```bash
uv venv
uv pip install -e .
```

## Configuration

The Jira MCP server requires the following environment variables:

- `JIRA_URL`: The base URL of your Jira instance (e.g., https://yourcompany.atlassian.net)
- `JIRA_USERNAME`: Your Jira username (email)
- `JIRA_API_TOKEN`: Your Jira API token

## Running

```bash
python -m jira
```