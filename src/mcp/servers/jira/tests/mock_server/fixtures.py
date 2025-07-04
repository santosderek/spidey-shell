"""
Pytest fixtures for mock Jira server.

This module provides fixtures for starting and stopping
a mock Jira server for tests.
"""

import asyncio
import logging
import pytest
from typing import AsyncGenerator, Generator

from jira_mcp.client import JiraTicketManager
from jira_mcp.settings import settings
from tests.utils.test_helpers import find_available_port, wait_for_server
from tests.mock_server.server_thread import start_server_in_thread
from tests.mock_server.server import app

logger = logging.getLogger(__name__)


@pytest.fixture(scope="session")
def mock_server_port():
    """Get a random available port for the mock server."""
    return find_available_port()


@pytest.fixture(scope="session")
def mock_server_thread(mock_server_port):
    """
    Start a mock Jira server in a background thread.
    
    This fixture is session-scoped for better performance.
    """
    # Start the server
    server_thread, server_url = start_server_in_thread(
        app=app,
        port=mock_server_port,
        timeout=10.0
    )
    
    # Yield the server thread and URL
    yield server_thread, server_url
    
    # Stop the server when session ends
    if not server_thread.stop_server(timeout=5.0):
        logger.warning("Server thread didn't exit gracefully")


@pytest.fixture(scope="session")
async def mock_jira_server(mock_server_thread) -> AsyncGenerator[str, None]:
    """
    Get the URL of the mock Jira server and ensure it's ready.
    
    Returns:
        Server URL when ready
    """
    # Unpack the server thread and URL
    _, server_url = mock_server_thread
    
    # Wait for server to become available
    is_ready = await wait_for_server(server_url, timeout=10)
    if not is_ready:
        pytest.fail(f"Server at {server_url} didn't become available within timeout")
    
    # Return the server URL
    yield server_url


@pytest.fixture
def mock_jira_manager(mock_jira_server) -> Generator[JiraTicketManager, None, None]:
    """
    Create a JiraTicketManager configured to use the mock server.
    
    This fixture is function-scoped for test isolation.
    
    Args:
        mock_jira_server: The mock server URL from the fixture
        
    Returns:
        A configured JiraTicketManager
    """
    # Create test credentials
    test_credentials = ("test_user", "test_password")
    
    # Save original settings
    original_url = settings.jira_url
    
    # Override settings for the test
    settings.jira_url = mock_jira_server
    
    # Create the manager with test credentials and timeout
    manager = JiraTicketManager(auth=test_credentials)
    
    # Verify server connection
    if not manager.jira_client:
        pytest.fail(f"Failed to connect to mock server at {mock_jira_server}")
    
    # Yield the manager for test use
    yield manager
    
    # Restore original settings
    settings.jira_url = original_url