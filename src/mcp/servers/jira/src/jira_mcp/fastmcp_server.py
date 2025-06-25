"""
FastMCP Server implementation for Jira MCP.

This module provides a FastMCP server implementation for Jira integration,
leveraging FastAPI for HTTP endpoints and the MCP protocol for standardized
AI communication.
"""

import logging
from typing import Dict, List, Optional, Any, Union, Literal

from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from mcp.server.fastmcp import FastMCP
from pydantic import BaseModel, Field

from jira_mcp.client import JiraTicketManager
from jira_mcp.formatters import ticket_to_markdown, generate_markdown_table

# Configure logging
logger = logging.getLogger(__name__)

# Create FastMCP and FastAPI applications
app = FastAPI(title="Jira MCP Server", description="MCP Server for Jira integration")

# Configure CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # In production, specify actual origins
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Create FastMCP server
mcp = FastMCP("JiraMCP", app=app)

# Define request and response models
class ListTicketsRequest(BaseModel):
    filter: Literal["assigned", "recent"] = Field(
        default="assigned", 
        description="Filter type: 'assigned' or 'recent'"
    )
    days: int = Field(
        default=30, 
        description="Number of days to look back (for recent filter)"
    )

class ViewTicketRequest(BaseModel):
    ticket_id: str = Field(..., description="ID of the ticket to view")

class Ticket(BaseModel):
    id: str
    title: str
    status: str
    url: str
    priority: Optional[str] = None
    assigned_to: Optional[str] = None
    description: Optional[str] = None
    created_at: Optional[str] = None
    updated_at: Optional[str] = None
    last_comment_at: Optional[str] = None

class ListTicketsResponse(BaseModel):
    tickets: List[Ticket]
    count: int
    markdown: str
    message: Optional[str] = None

class ViewTicketResponse(BaseModel):
    ticket: Ticket
    markdown: str
    
class ViewCommentsResponse(BaseModel):
    ticket_id: str
    comments: List[str]
    count: int
    markdown: str
    message: Optional[str] = None

class ErrorResponse(BaseModel):
    error: str


# MCP Tools implementation with FastMCP decorators
@mcp.tool()
def list_assigned_tickets(args: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
    """
    List tickets assigned to the current user.
    
    Returns a list of tickets assigned to the current user, formatted as a markdown table.
    """
    args = args or {}
    
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


@mcp.tool()
def list_recent_tickets(args: Dict[str, Any]) -> Dict[str, Any]:
    """
    List recently updated tickets.
    
    Args:
        args: Dictionary containing parameters:
            - days: Number of days to look back (default: 30)
            
    Returns a list of tickets updated within the specified number of days,
    formatted as a markdown table.
    """
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

    manager = JiraTicketManager()
    ticket = manager.get_ticket_by_id(ticket_id)

    if not ticket:
        raise ValueError(f"Ticket not found: {ticket_id}")

    markdown = ticket_to_markdown(ticket)
    return {"ticket": ticket, "markdown": markdown}


@mcp.tool()
def view_ticket_comments(args: Dict[str, Any]) -> Dict[str, Any]:
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


# FastAPI routes
@app.get("/")
async def root():
    """Root endpoint providing server information."""
    return {
        "name": "Jira MCP Server",
        "version": "0.1.0",
        "description": "MCP Server for Jira integration",
        "endpoints": [
            "/tickets/assigned",
            "/tickets/recent",
            "/tickets/{ticket_id}",
            "/tickets/{ticket_id}/comments",
        ],
    }


@app.get("/tickets/assigned", response_model=ListTicketsResponse)
async def get_assigned_tickets():
    """Get tickets assigned to the current user."""
    try:
        return list_assigned_tickets({})
    except Exception as e:
        logger.error(f"Error fetching assigned tickets: {e}")
        raise HTTPException(status_code=500, detail=str(e))


@app.get("/tickets/recent", response_model=ListTicketsResponse)
async def get_recent_tickets(days: int = 30):
    """Get tickets updated within the specified number of days."""
    try:
        return list_recent_tickets({"days": days})
    except Exception as e:
        logger.error(f"Error fetching recent tickets: {e}")
        raise HTTPException(status_code=500, detail=str(e))


@app.get("/tickets/{ticket_id}", response_model=ViewTicketResponse)
async def get_ticket(ticket_id: str):
    """Get details of a specific ticket."""
    try:
        return view_ticket({"ticket_id": ticket_id})
    except ValueError as e:
        logger.warning(f"Ticket error: {e}")
        raise HTTPException(status_code=404, detail=str(e))
    except Exception as e:
        logger.error(f"Error fetching ticket {ticket_id}: {e}")
        raise HTTPException(status_code=500, detail=str(e))


@app.get("/tickets/{ticket_id}/comments", response_model=ViewCommentsResponse)
async def get_ticket_comments(ticket_id: str):
    """Get comments on a specific ticket."""
    try:
        return view_ticket_comments({"ticket_id": ticket_id})
    except ValueError as e:
        logger.warning(f"Comments error: {e}")
        raise HTTPException(status_code=404, detail=str(e))
    except Exception as e:
        logger.error(f"Error fetching comments for ticket {ticket_id}: {e}")
        raise HTTPException(status_code=500, detail=str(e))


# MCP Server instances
mcp_server = mcp

def run_server(host="0.0.0.0", port=8000, log_level="info"):
    """
    Run the FastAPI server.
    
    Args:
        host: Host to bind to
        port: Port to listen on
        log_level: Logging level (debug, info, warning, error, critical)
    """
    import uvicorn
    uvicorn.run("jira_mcp.fastmcp_server:app", host=host, port=port, log_level=log_level)