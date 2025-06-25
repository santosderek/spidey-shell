"""
MCP Tool implementations for Jira integration.

This module provides MCP tools that integrate with Jira API.
It contains both standard MCP tool functions and the FastMCP decorators
for use with FastAPI.
"""

import logging
from typing import Dict, List, Any, Optional, Union

from jira_mcp.client import JiraTicketManager
from jira_mcp.formatters import generate_markdown_table, ticket_to_markdown

logger = logging.getLogger(__name__)


# MCP Tool: List Assigned Tickets
def mcp_list_assigned_tickets(args: Dict[str, Any]) -> Dict[str, Any]:
    """MCP tool to list tickets assigned to the current user."""
    manager = JiraTicketManager()
    tickets = manager.get_assigned_tickets()

    if not tickets:
        return {
            "message": "No assigned tickets found",
            "tickets": [],
            "count": 0,
            "markdown": "No tickets found."
        }

    markdown_table = generate_markdown_table(tickets)
    return {"tickets": tickets, "count": len(tickets), "markdown": markdown_table}


# MCP Tool: List Recent Tickets
def mcp_list_recent_tickets(args: Dict[str, Any]) -> Dict[str, Any]:
    """MCP tool to list recently updated tickets."""
    days = int(args.get("days", 30))
    manager = JiraTicketManager()
    tickets = manager.get_recent_tickets(days)

    if not tickets:
        return {
            "message": f"No tickets updated in the last {days} days",
            "tickets": [],
            "count": 0,
            "markdown": "No tickets found."
        }

    markdown_table = generate_markdown_table(tickets)
    return {"tickets": tickets, "count": len(tickets), "markdown": markdown_table}


# MCP Tool: View Ticket Details
def mcp_view_ticket(args: Dict[str, Any]) -> Dict[str, Any]:
    """MCP tool to view details of a specific ticket."""
    ticket_id = args.get("ticket_id")
    if not ticket_id:
        raise ValueError("No ticket ID provided")

    manager = JiraTicketManager()
    ticket = manager.get_ticket_by_id(ticket_id)

    if not ticket:
        raise ValueError(f"Ticket not found: {ticket_id}")

    markdown = ticket_to_markdown(ticket)
    return {"ticket": ticket, "markdown": markdown}


# MCP Tool: View Ticket Comments
def mcp_view_ticket_comments(args: Dict[str, Any]) -> Dict[str, Any]:
    """MCP tool to view comments on a specific ticket."""
    ticket_id = args.get("ticket_id")
    if not ticket_id:
        raise ValueError("No ticket ID provided")

    manager = JiraTicketManager()
    comments = manager.get_ticket_comments(ticket_id)

    if not comments:
        return {
            "message": f"No comments found for ticket: {ticket_id}",
            "ticket_id": ticket_id,
            "comments": [],
            "count": 0,
            "markdown": "No comments found."
        }

    return {
        "ticket_id": ticket_id,
        "comments": comments,
        "count": len(comments),
        "markdown": "\n---\n".join(comments),
    }


# MCP Tool schema/registry
MCP_TOOLS = {
    "jira.list-assigned-tickets": {
        "function": mcp_list_assigned_tickets,
        "description": "List tickets assigned to the current user",
        "args": {},
    },
    "jira.list-recent-tickets": {
        "function": mcp_list_recent_tickets,
        "description": "List recently updated tickets",
        "args": {
            "days": {
                "type": "integer",
                "default": 30,
                "description": "Number of days to look back",
            }
        },
    },
    "jira.view-ticket": {
        "function": mcp_view_ticket,
        "description": "View details of a specific ticket",
        "args": {
            "ticket_id": {
                "type": "string",
                "required": True,
                "description": "Jira ticket ID",
            }
        },
    },
    "jira.view-ticket-comments": {
        "function": mcp_view_ticket_comments,
        "description": "View comments on a specific ticket",
        "args": {
            "ticket_id": {
                "type": "string",
                "required": True,
                "description": "Jira ticket ID",
            }
        },
    },
}