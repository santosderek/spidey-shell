"""
API endpoint implementations for the mock Jira server.

This module provides FastAPI route handlers for the various
Jira API endpoints required by the JIRA client.
"""

from datetime import datetime
from typing import List, Dict, Any, Optional

from fastapi import APIRouter, Depends, HTTPException, Path, Query, Request
from fastapi.security import HTTPBasic, HTTPBasicCredentials
from fastapi.responses import JSONResponse

from tests.mock_server.models import (
    ServerInfo, 
    ServerHealthCheck,
    UserDetails,
    FieldDefinition,
    FieldSchema,
    ProjectDetails,
    AvatarUrls,
    ProjectLead,
    ProjectCategory
)

# Create API router
api_router = APIRouter(prefix="/rest/api/2")

# Setup basic auth
security = HTTPBasic(auto_error=False)


@api_router.get("/serverInfo", response_model=ServerInfo)
async def get_server_info(request: Request):
    """
    Get information about the Jira server.
    
    This endpoint is used by the JIRA client during initialization
    to verify server connectivity and version.
    """
    # Get the base URL from the request
    base_url = str(request.base_url).rstrip('/')
    
    # Current time in ISO format
    current_time = datetime.now().strftime("%Y-%m-%dT%H:%M:%S.000%z")
    
    return ServerInfo(
        baseUrl=base_url,
        version="9.0.0",
        versionNumbers=[9, 0, 0],
        buildNumber=90000,
        buildDate=current_time,
        serverTime=current_time,
        scmInfo="mock-1234",
        serverTitle="Mock Jira Server",
        healthChecks=[
            ServerHealthCheck(
                name="SystemInfo",
                description="Server health check",
                passed=True
            )
        ]
    )


@api_router.get("/myself", response_model=UserDetails)
async def get_current_user(request: Request):
    """
    Get information about the authenticated user.
    
    This endpoint is used by the JIRA client to get information
    about the current user after authentication.
    """
    # Get the base URL from the request
    base_url = str(request.base_url).rstrip('/')
    
    return UserDetails(
        self=f"{base_url}/rest/api/2/user?accountId=mock-user",
        accountId="mock-user",
        emailAddress="test@example.com",
        displayName="Test User"
    )


@api_router.get("/field", response_model=List[FieldDefinition])
async def get_fields():
    """
    Get all field definitions.
    
    This endpoint is used by the JIRA client to get information
    about the available fields for issues.
    """
    return [
        FieldDefinition(
            id="summary",
            name="Summary",
            schema=FieldSchema(type="string")
        ),
        FieldDefinition(
            id="description",
            name="Description",
            schema=FieldSchema(type="string")
        ),
        FieldDefinition(
            id="status",
            name="Status",
            schema=FieldSchema(type="status")
        ),
        FieldDefinition(
            id="priority",
            name="Priority",
            schema=FieldSchema(type="priority")
        ),
        FieldDefinition(
            id="assignee",
            name="Assignee",
            schema=FieldSchema(type="user")
        ),
        FieldDefinition(
            id="created",
            name="Created",
            schema=FieldSchema(type="datetime")
        ),
        FieldDefinition(
            id="updated",
            name="Updated",
            schema=FieldSchema(type="datetime")
        )
    ]


@api_router.get("/project", response_model=List[ProjectDetails])
async def get_projects(request: Request):
    """
    Get all projects.
    
    This endpoint is used by the JIRA client to get information
    about the available projects.
    """
    # Get the base URL from the request
    base_url = str(request.base_url).rstrip('/')
    
    return [
        ProjectDetails(
            self=f"{base_url}/rest/api/2/project/TEST",
            id="10000",
            key="TEST",
            name="Test Project",
            projectTypeKey="software",
            avatarUrls=AvatarUrls(
                x48=f"{base_url}/secure/projectavatar?size=large&pid=10000",
                x24=f"{base_url}/secure/projectavatar?size=small&pid=10000",
                x16=f"{base_url}/secure/projectavatar?size=xsmall&pid=10000",
                x32=f"{base_url}/secure/projectavatar?size=medium&pid=10000"
            ),
            lead=ProjectLead(
                self=f"{base_url}/rest/api/2/user?accountId=mock-user",
                accountId="mock-user",
                displayName="Test User",
                active=True
            )
        )
    ]


@api_router.get("/issue/{issue_key}")
async def get_issue(issue_key: str, request: Request):
    """
    Get a specific issue.
    
    This endpoint is used by the JIRA client to get information
    about a specific issue.
    """
    # Get the base URL from the request
    base_url = str(request.base_url).rstrip('/')
    
    # Check if the issue exists in our mock data
    issue_id = None
    if "-" in issue_key:
        parts = issue_key.split("-")
        if len(parts) == 2 and parts[0] == "TEST" and parts[1].isdigit():
            issue_id = parts[1]
    
    if issue_id and issue_id.isdigit() and 1 <= int(issue_id) <= 5:
        # Issue exists
        user = UserDetails(
            self=f"{base_url}/rest/api/2/user?accountId=mock-user",
            accountId="mock-user",
            emailAddress="test@example.com",
            displayName="Test User"
        )
        
        comments = []
        if int(issue_id) % 2 == 0:  # Even numbered issues have comments
            for j in range(1, 3):
                comment_date = datetime.now().strftime("%Y-%m-%dT%H:%M:%S.000Z")
                comments.append({
                    "id": f"comment-{issue_id}-{j}",
                    "author": {
                        "accountId": "mock-user",
                        "displayName": "Test User",
                        "emailAddress": "test@example.com"
                    },
                    "body": f"Test comment {j} for issue {issue_key}",
                    "created": comment_date,
                    "updated": comment_date
                })
        
        created = datetime.now().strftime("%Y-%m-%dT%H:%M:%S.000Z")
        updated = datetime.now().strftime("%Y-%m-%dT%H:%M:%S.000Z")
        
        return {
            "id": issue_id,
            "key": issue_key,
            "fields": {
                "summary": f"Test Issue {issue_id}",
                "description": f"Description for test issue {issue_id}",
                "created": created,
                "updated": updated,
                "status": {"name": "Open" if int(issue_id) % 3 == 1 else "In Progress" if int(issue_id) % 3 == 2 else "Done"},
                "priority": {"name": ["Highest", "High", "Medium", "Low", "Lowest"][int(issue_id) % 5]},
                "assignee": user if int(issue_id) % 2 == 0 else None,
                "comment": {"comments": comments} if comments else None
            }
        }
    else:
        # Issue doesn't exist
        raise HTTPException(status_code=404, detail=f"Issue {issue_key} not found")


async def _process_search_request(request: Request, jql: str, max_results: int, start_at: int):
    """
    Process a search request, common code for both GET and POST methods.
    """
    # Get the base URL from the request
    base_url = str(request.base_url).rstrip('/')
    
    # Create mock issues based on the JQL
    issues = []
    for i in range(1, 6):
        issue_key = f"TEST-{i}"
        created = datetime.now().strftime("%Y-%m-%dT%H:%M:%S.000Z")
        updated = datetime.now().strftime("%Y-%m-%dT%H:%M:%S.000Z")
        
        # Simple filtering based on JQL
        include = True
        if "assignee = currentUser()" in jql and i % 2 != 0:
            include = False
            
        if include:
            user = {
                "self": f"{base_url}/rest/api/2/user?accountId=mock-user",
                "accountId": "mock-user",
                "emailAddress": "test@example.com",
                "displayName": "Test User",
                "active": True
            }
            
            comments = []
            if i % 2 == 0:  # Even numbered issues have comments
                for j in range(1, 3):
                    comment_date = datetime.now().strftime("%Y-%m-%dT%H:%M:%S.000Z")
                    comments.append({
                        "id": f"comment-{i}-{j}",
                        "author": user,
                        "body": f"Test comment {j} for issue {issue_key}",
                        "created": comment_date,
                        "updated": comment_date
                    })
            
            issue = {
                "id": str(i),
                "key": issue_key,
                "self": f"{base_url}/rest/api/2/issue/{issue_key}",
                "fields": {
                    "summary": f"Test Issue {i}",
                    "description": f"Description for test issue {i}",
                    "created": created,
                    "updated": updated,
                    "status": {"name": "Open" if i % 3 == 1 else "In Progress" if i % 3 == 2 else "Done"},
                    "priority": {"name": ["Highest", "High", "Medium", "Low", "Lowest"][i % 5]},
                    "assignee": user if i % 2 == 0 else None,
                    "comment": {"comments": comments} if comments else None
                }
            }
            issues.append(issue)
    
    # Sort results if requested
    if "ORDER BY" in jql:
        if "ORDER BY updated" in jql:
            # No need to sort for mock data
            pass
    
    # Paginate results
    total = len(issues)
    issues = issues[start_at:start_at + max_results]
    
    return {
        "startAt": start_at,
        "maxResults": max_results,
        "total": total,
        "issues": issues
    }


@api_router.get("/search")
async def search_issues_get(
    request: Request,
    jql: str = "",
    maxResults: int = 50,
    startAt: int = 0,
    validateQuery: bool = True,
    fields: str = "*all"
):
    """
    Search for issues using JQL (GET method).
    
    This endpoint is used by the JIRA client to search for issues
    using JQL queries via GET request.
    """
    return await _process_search_request(request, jql, maxResults, startAt)


@api_router.post("/search")
async def search_issues_post(request: Request):
    """
    Search for issues using JQL (POST method).
    
    This endpoint is used by the JIRA client to search for issues
    using JQL queries via POST request.
    """
    # Parse the request body
    body = await request.json()
    jql = body.get("jql", "")
    max_results = body.get("maxResults", 50)
    start_at = body.get("startAt", 0)
    
    return await _process_search_request(request, jql, max_results, start_at)