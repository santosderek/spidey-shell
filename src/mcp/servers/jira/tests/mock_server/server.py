"""
Mock Jira API server implementation for testing.

This module provides a FastAPI-based mock server that simulates
the Jira API for testing. It provides endpoints that match the
real Jira API but with in-memory storage.
"""

import asyncio
import json
import logging
import uuid
from contextlib import asynccontextmanager
from datetime import datetime, timedelta
from typing import Dict, List, Optional, Any, Set

import uvicorn
from fastapi import FastAPI, HTTPException, WebSocket, WebSocketDisconnect, Depends
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel

from tests.mock_server.api import api_router

# Setup logger
logger = logging.getLogger(__name__)


# WebSocket connection manager
class ConnectionManager:
    """Manages WebSocket connections for real-time updates."""
    
    def __init__(self):
        """Initialize with an empty set of connections."""
        self.active_connections: Set[WebSocket] = set()
    
    async def connect(self, websocket: WebSocket):
        """Accept and register a new WebSocket connection."""
        await websocket.accept()
        self.active_connections.add(websocket)
        logger.debug(f"WebSocket client connected. Active connections: {len(self.active_connections)}")
    
    def disconnect(self, websocket: WebSocket):
        """Remove a WebSocket connection."""
        self.active_connections.remove(websocket)
        logger.debug(f"WebSocket client disconnected. Active connections: {len(self.active_connections)}")
    
    async def broadcast(self, data: Dict[str, Any]):
        """Broadcast data to all connected clients."""
        dead_connections = set()
        
        for connection in self.active_connections:
            try:
                await connection.send_json(data)
            except Exception as e:
                logger.warning(f"Failed to send message to client: {e}")
                dead_connections.add(connection)
        
        # Remove dead connections
        for dead in dead_connections:
            self.active_connections.remove(dead)


# Models for our mock server
class JiraUser(BaseModel):
    """Simplified Jira user model."""
    accountId: str
    displayName: str
    emailAddress: str


class JiraComment(BaseModel):
    """Simplified Jira comment model."""
    id: str
    author: JiraUser
    body: str
    created: str
    updated: str


class JiraPriority(BaseModel):
    """Simplified Jira priority model."""
    id: str
    name: str


class JiraStatus(BaseModel):
    """Simplified Jira status model."""
    id: str
    name: str


class JiraIssueFields(BaseModel):
    """Simplified Jira issue fields model."""
    summary: str
    description: Optional[str] = None
    created: str
    updated: str
    status: JiraStatus
    priority: Optional[JiraPriority] = None
    assignee: Optional[JiraUser] = None
    comment: Optional[Dict[str, List[JiraComment]]] = None


class JiraIssue(BaseModel):
    """Simplified Jira issue model."""
    id: str
    key: str
    fields: JiraIssueFields


# In-memory database
class JiraMockDB:
    """In-memory database for mock Jira server."""
    
    def __init__(self):
        """Initialize the mock database."""
        self.users: Dict[str, JiraUser] = {}
        self.issues: Dict[str, JiraIssue] = {}
        self.priorities = {
            "1": JiraPriority(id="1", name="Highest"),
            "2": JiraPriority(id="2", name="High"),
            "3": JiraPriority(id="3", name="Medium"),
            "4": JiraPriority(id="4", name="Low"),
            "5": JiraPriority(id="5", name="Lowest"),
        }
        self.statuses = {
            "1": JiraStatus(id="1", name="Open"),
            "2": JiraStatus(id="2", name="In Progress"),
            "3": JiraStatus(id="3", name="Done"),
        }
        
        # Initialize with default data
        self._initialize_data()
    
    def _initialize_data(self):
        """Initialize with default testing data."""
        # Create test user
        self.users["user1"] = JiraUser(
            accountId="user1",
            displayName="Test User",
            emailAddress="test@example.com"
        )
        
        # Create sample issues
        for i in range(1, 6):
            key = f"TEST-{i}"
            created = (datetime.now() - timedelta(days=10-i)).strftime("%Y-%m-%dT%H:%M:%S.000Z")
            updated = (datetime.now() - timedelta(days=5-i)).strftime("%Y-%m-%dT%H:%M:%S.000Z")
            
            comments = []
            if i % 2 == 0:  # Add comments to even-numbered issues
                for j in range(1, 3):
                    comment_date = (datetime.now() - timedelta(days=3-j)).strftime("%Y-%m-%dT%H:%M:%S.000Z")
                    comments.append(JiraComment(
                        id=f"comment-{i}-{j}",
                        author=self.users["user1"],
                        body=f"Test comment {j} for issue {key}",
                        created=comment_date,
                        updated=comment_date
                    ))
            
            status_id = str((i % 3) + 1)
            priority_id = str((i % 5) + 1)
            
            fields = JiraIssueFields(
                summary=f"Test Issue {i}",
                description=f"Description for test issue {i}",
                created=created,
                updated=updated,
                status=self.statuses[status_id],
                priority=self.priorities[priority_id],
                assignee=self.users["user1"] if i % 2 == 0 else None,
                comment={"comments": comments} if comments else None
            )
            
            issue = JiraIssue(
                id=str(i),
                key=key,
                fields=fields
            )
            
            self.issues[key] = issue


# Create global connection manager instance
manager = ConnectionManager()


# Create lifespan context manager
@asynccontextmanager
async def lifespan(app: FastAPI):
    """
    Lifespan manager for FastAPI server.
    
    Sets up and tears down resources for the application.
    """
    # Create the database on startup
    app.state.db = JiraMockDB()
    logger.info("Mock Jira server started with test data")
    yield
    # Clean up on shutdown
    manager.active_connections.clear()
    logger.info("Mock Jira server shut down")


# Create the FastAPI application
app = FastAPI(
    title="Mock Jira Server",
    description="Mock Jira API for testing",
    version="1.0.0",
    lifespan=lifespan
)

# Add CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Include API router for Jira endpoints
app.include_router(api_router)


# Dependency to get the database (for HTTP endpoints)
def get_db(app=app):
    """Get the database from the app state."""
    return app.state.db


# WebSocket endpoint for live updates
@app.websocket("/jira/ws")
async def websocket_endpoint(websocket: WebSocket):
    """WebSocket endpoint for real-time updates."""
    await manager.connect(websocket)
    try:
        while True:
            # Wait for messages from the client
            data = await websocket.receive_text()
            # Echo back the message (real implementation would do more)
            await websocket.send_text(f"Echo: {data}")
    except WebSocketDisconnect:
        manager.disconnect(websocket)


# API endpoints
@app.get("/rest/api/3/issue/{issue_key}", response_model=JiraIssue)
async def get_issue(issue_key: str, db: JiraMockDB = Depends(get_db)):
    """Get a specific issue by key."""
    issue = db.issues.get(issue_key)
    if not issue:
        raise HTTPException(status_code=404, detail=f"Issue {issue_key} not found")
    return issue


@app.get("/rest/api/3/search")
async def search_issues(
    jql: str, 
    db: JiraMockDB = Depends(get_db),
    maxResults: int = 50,
    startAt: int = 0
):
    """Search issues using JQL."""
    # Simple JQL parser - in a real implementation, this would be more sophisticated
    issues = list(db.issues.values())
    
    # Handle assignee filter
    if "assignee = currentUser()" in jql:
        issues = [issue for issue in issues if issue.fields.assignee is not None]
    
    # Handle updated date filter
    if "updated >=" in jql:
        try:
            # Extract date from JQL
            date_str = jql.split("updated >= ")[1].split(" ")[0]
            filter_date = datetime.strptime(date_str, "%Y-%m-%d")
            
            # Filter by date
            issues = [
                issue for issue in issues 
                if datetime.strptime(issue.fields.updated.split(".")[0], "%Y-%m-%dT%H:%M:%S") >= filter_date
            ]
        except Exception as e:
            logger.error(f"Error parsing JQL date filter: {e}")
    
    # Sort by updated date if specified
    if "ORDER BY updated" in jql:
        issues.sort(
            key=lambda issue: datetime.strptime(issue.fields.updated.split(".")[0], "%Y-%m-%dT%H:%M:%S"),
            reverse="DESC" in jql
        )
    
    # Apply pagination
    total = len(issues)
    issues = issues[startAt:startAt + maxResults]
    
    return {
        "startAt": startAt,
        "maxResults": maxResults,
        "total": total,
        "issues": issues
    }


# Health check endpoint
@app.get("/health")
async def health():
    """Health check endpoint."""
    return {"status": "ok"}


# These functions are now in server_thread.py
# Left empty to maintain backwards compatibility
# New code should use server_thread.py directly