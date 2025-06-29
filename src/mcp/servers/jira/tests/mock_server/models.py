"""
Response models for the mock Jira server.

This module defines Pydantic models for the various responses
returned by the mock Jira server endpoints.
"""

from typing import Dict, List, Optional, Any
from datetime import datetime
from pydantic import BaseModel, Field


class ServerHealthCheck(BaseModel):
    """Health check item in server info."""
    
    name: str
    description: str
    passed: bool


class ServerInfo(BaseModel):
    """Response model for /rest/api/2/serverInfo."""
    
    baseUrl: str
    version: str
    versionNumbers: List[int]
    deploymentType: str = "Server"
    buildNumber: int
    buildDate: str
    serverTime: str
    scmInfo: str
    serverTitle: str
    healthChecks: List[ServerHealthCheck]


class UserDetails(BaseModel):
    """Response model for /rest/api/2/myself."""
    
    self: str
    accountId: str
    emailAddress: str
    displayName: str
    active: bool = True
    timeZone: str = "UTC"
    locale: str = "en_US"
    groups: Dict[str, List[str]] = Field(default_factory=lambda: {"size": 1, "items": []})
    applicationRoles: Dict[str, List[str]] = Field(default_factory=lambda: {"size": 1, "items": []})


class FieldSchema(BaseModel):
    """Schema for a field definition."""
    
    type: str
    items: Optional[str] = None
    system: Optional[str] = None
    custom: Optional[str] = None
    customId: Optional[int] = None


class FieldDefinition(BaseModel):
    """Response model for items in /rest/api/2/field."""
    
    id: str
    key: Optional[str] = None
    name: str
    schema: FieldSchema


class ProjectCategory(BaseModel):
    """Project category details."""
    
    self: str
    id: str
    name: str
    description: Optional[str] = None


class ProjectLead(BaseModel):
    """Project lead details."""
    
    self: str
    accountId: str
    displayName: str
    active: bool


class AvatarUrls(BaseModel):
    """Avatar URLs for various sizes."""
    
    x48: str
    x24: str
    x16: str
    x32: str


class ProjectDetails(BaseModel):
    """Response model for items in /rest/api/2/project."""
    
    self: str
    id: str
    key: str
    name: str
    projectTypeKey: str
    simplified: bool = True
    avatarUrls: AvatarUrls
    projectCategory: Optional[ProjectCategory] = None
    lead: Optional[ProjectLead] = None