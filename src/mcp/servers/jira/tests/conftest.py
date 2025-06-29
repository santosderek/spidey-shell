"""
Pytest fixtures for Jira MCP testing.

This module provides fixtures that can be used across all tests,
including the mock Jira server fixture.
"""

import asyncio
import os
import logging
from typing import Dict, Any, Generator, Tuple, AsyncGenerator

import pytest
import httpx
from fastapi.testclient import TestClient

from jira_mcp.client import JiraTicketManager
from jira_mcp.settings import settings

# Import fixtures from their respective modules
from tests.mock_server.fixtures import (
    mock_server_port,
    mock_server_thread,
    mock_jira_server,
    mock_jira_manager,
)

# Setup logger
logger = logging.getLogger(__name__)


# Fixture for sample ticket data
@pytest.fixture
def sample_tickets() -> Dict[str, Any]:
    """
    Provide sample ticket data for tests.
    
    Returns:
        Dictionary with sample tickets
    """
    return {
        "TEST-1": {
            "id": "TEST-1",
            "title": "Test Issue 1",
            "status": "Open",
            "url": "http://example.com/browse/TEST-1",
            "priority": "Highest",
            "description": "Description for test issue 1",
        },
        "TEST-2": {
            "id": "TEST-2",
            "title": "Test Issue 2",
            "status": "In Progress",
            "url": "http://example.com/browse/TEST-2",
            "priority": "High",
            "description": "Description for test issue 2",
            "assigned_to": "Test User",
        },
    }


# Fixture for memory-based cache (to avoid disk writes during tests)
@pytest.fixture
def memory_cache():
    """
    Create an in-memory cache for tests.
    
    This fixture ensures that tests don't write to disk.
    """
    cache = {}
    return cache


# Fixture for patching environment variables
@pytest.fixture
def mock_env_vars(monkeypatch):
    """
    Mock environment variables for testing.
    
    Args:
        monkeypatch: Pytest monkeypatch fixture
        
    Returns:
        Function to set mock environment variables
    """
    def _set_env_vars(env_vars: Dict[str, str]):
        for key, value in env_vars.items():
            monkeypatch.setenv(key, value)
    
    return _set_env_vars