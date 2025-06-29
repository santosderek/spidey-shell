"""
Pydantic schemas for Jira MCP server.

This module contains all the request and response models used by the
Jira MCP server for type validation and serialization.
"""

from typing import List, Optional, Literal
from pydantic import BaseModel, Field


# Request models
class ListTicketsRequest(BaseModel):
    filter: Literal["assigned", "recent"] = Field(
        default="assigned", description="Filter type: 'assigned' or 'recent'"
    )
    days: int = Field(
        default=30, description="Number of days to look back (for recent filter)"
    )


class ViewTicketRequest(BaseModel):
    ticket_id: str = Field(..., description="ID of the ticket to view")


# Data models
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


# Response models
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


# Export all models for easy importing
__all__ = [
    "ListTicketsRequest",
    "ViewTicketRequest",
    "Ticket",
    "ListTicketsResponse",
    "ViewTicketResponse",
    "ViewCommentsResponse",
    "ErrorResponse",
]

