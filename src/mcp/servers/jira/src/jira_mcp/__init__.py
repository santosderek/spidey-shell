"""
Jira MCP package for integrating Jira with the MCP framework.

This package provides MCP tools for interacting with Jira.
"""

__version__ = "0.1.0"

from jira_mcp.client import JiraTicketManager
from jira_mcp.tools import (
    mcp_list_assigned_tickets,
    mcp_list_recent_tickets,
    mcp_view_ticket,
    mcp_view_ticket_comments,
    MCP_TOOLS,
)
from jira_mcp.formatters import format_date, generate_markdown_table, ticket_to_markdown

__all__ = [
    "JiraTicketManager",
    "mcp_list_assigned_tickets",
    "mcp_list_recent_tickets",
    "mcp_view_ticket",
    "mcp_view_ticket_comments",
    "MCP_TOOLS",
    "format_date",
    "generate_markdown_table",
    "ticket_to_markdown",
]