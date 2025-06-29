"""
Utility functions for test execution.

This module contains helper functions used across tests,
such as finding available ports, waiting for services to be ready,
and generating test data.
"""

import socket
import time
import asyncio
from contextlib import closing
from typing import Optional

import httpx


def find_available_port() -> int:
    """
    Find a random available port on localhost.
    
    Returns:
        An available port number
    """
    with closing(socket.socket(socket.AF_INET, socket.SOCK_STREAM)) as s:
        s.bind(('', 0))
        s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        return s.getsockname()[1]


async def wait_for_server(url: str, timeout: int = 10, path: str = "/health") -> bool:
    """
    Wait for a server to become available by polling its health endpoint.
    
    Args:
        url: The base URL of the server
        timeout: Maximum seconds to wait
        path: Path to health endpoint
        
    Returns:
        True if server became available, False if timeout was reached
    """
    health_url = f"{url}{path}"
    end_time = time.time() + timeout
    
    while time.time() < end_time:
        try:
            async with httpx.AsyncClient() as client:
                response = await client.get(health_url, timeout=2.0)
                if response.status_code == 200:
                    return True
        except (httpx.ConnectError, httpx.ReadTimeout):
            pass
            
        await asyncio.sleep(0.5)
    
    return False


def generate_test_issue(issue_id: str, summary: str, status: str = "Open") -> dict:
    """
    Generate test issue data for mocking.
    
    Args:
        issue_id: The ID of the issue (e.g. TEST-123)
        summary: The issue summary/title
        status: The issue status
        
    Returns:
        A dictionary mimicking a Jira issue structure
    """
    return {
        "id": issue_id.split("-")[1],
        "key": issue_id,
        "fields": {
            "summary": summary,
            "description": f"Description for {issue_id}",
            "created": "2023-01-01T10:00:00.000Z",
            "updated": "2023-01-02T11:00:00.000Z",
            "status": {"name": status},
            "priority": {"name": "Medium"}
        }
    }