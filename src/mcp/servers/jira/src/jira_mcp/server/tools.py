from typing import Optional, Dict, Any, cast

from jira_mcp.client import JiraTicketManager
from jira_mcp.formatters import generate_markdown_table, ticket_to_markdown
from jira_mcp.settings import settings

from . import mcp


# Create a singleton ticket manager instance
_ticket_manager = None


def get_ticket_manager() -> JiraTicketManager:
    """
    Get or create a JiraTicketManager instance.
    
    Returns:
        A JiraTicketManager instance with proper authentication.
        
    Raises:
        ValueError: If no credentials are configured.
    """
    global _ticket_manager
    
    if _ticket_manager is None:
        if not settings.has_credentials:
            raise ValueError(
                "Jira credentials not configured. Please set JIRA_MCP_JIRA_USERNAME and JIRA_MCP_JIRA_API_TOKEN."
            )
        _ticket_manager = JiraTicketManager()
        
    return _ticket_manager


# MCP Tools implementation with FastMCP decorators
@mcp.tool()
def list_tickets(args: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
    """
    List Jira tickets based on filter criteria.

    Args:
        args: Dictionary containing parameters:
            - filter: Filter type ('assigned' or 'recent', default: 'assigned')
            - days: Number of days to look back for recent tickets (default: 30)

    Returns a list of tickets matching the filter, formatted as a markdown table.
    """
    args = args or {}
    filter_type = args.get("filter", "assigned")
    days = int(args.get("days", settings.default_days_lookback))
    
    manager = get_ticket_manager()
    
    if filter_type == "recent":
        tickets = manager.get_recent_tickets(days)
        no_tickets_message = f"No tickets updated in the last {days} days"
    else:  # assigned
        tickets = manager.get_assigned_tickets()
        no_tickets_message = "No assigned tickets found"
    
    if not tickets:
        return {
            "message": no_tickets_message,
            "tickets": [],
            "count": 0,
            "markdown": "No tickets found.",
        }

    markdown_table = generate_markdown_table(tickets)
    return {
        "tickets": tickets, 
        "count": len(tickets), 
        "markdown": markdown_table,
        "message": f"Found {len(tickets)} tickets"
    }


@mcp.tool()
def view_ticket(args: Dict[str, Any]) -> Dict[str, Any]:
    """
    View details of a specific ticket.

    Args:
        args: Dictionary containing parameters:
            - ticket_id: ID of the ticket to view (required)

    Returns detailed information about the specified ticket formatted in markdown.
    """
    ticket_id = args.get("ticket_id")
    if not ticket_id:
        raise ValueError("No ticket ID provided")

    manager = get_ticket_manager()
    ticket = manager.get_ticket_by_id(ticket_id)

    if not ticket:
        raise ValueError(f"Ticket not found: {ticket_id}")

    markdown = ticket_to_markdown(ticket)
    return {"ticket": ticket, "markdown": markdown}


@mcp.tool()
def show_ticket_comments(args: Dict[str, Any]) -> Dict[str, Any]:
    """
    View comments on a specific ticket.

    Args:
        args: Dictionary containing parameters:
            - ticket_id: ID of the ticket to view comments for (required)

    Returns a list of comments on the specified ticket, formatted in markdown.
    """
    ticket_id = args.get("ticket_id")
    if not ticket_id:
        raise ValueError("No ticket ID provided")

    manager = get_ticket_manager()
    comments = manager.get_ticket_comments(ticket_id)

    if not comments:
        return {
            "message": f"No comments found for ticket: {ticket_id}",
            "ticket_id": ticket_id,
            "comments": [],
            "count": 0,
            "markdown": "No comments found.",
        }

    return {
        "ticket_id": ticket_id,
        "comments": comments,
        "count": len(comments),
        "markdown": "\n---\n".join(comments),
        "message": f"Found {len(comments)} comments for ticket {ticket_id}"
    }
