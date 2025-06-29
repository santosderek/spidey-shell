"""
Pydantic schemas for Jira mock server.

This module contains the request and response models used by the
Jira mock server tools.
"""

from typing import Optional, Literal
from pydantic import BaseModel, Field


class ServerStatus(BaseModel):
    """Status of the mock server."""
    status: Literal["running", "stopped", "error"]
    url: Optional[str] = None
    message: str
    markdown: str


class StartServerRequest(BaseModel):
    """Parameters for starting a mock server."""
    host: str = Field(default="0.0.0.0", description="Host to bind to")
    port: int = Field(default=8000, description="Port to listen on")


class StartServerResponse(BaseModel):
    """Response from starting a mock server."""
    status: Literal["running", "error"]
    url: Optional[str] = None
    message: str
    markdown: str


# Export models
__all__ = [
    "ServerStatus",
    "StartServerRequest",
    "StartServerResponse",
]