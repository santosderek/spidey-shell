"""
Tests for the FastMCP server implementation.
"""
import json
from unittest.mock import patch, MagicMock
from fastapi.testclient import TestClient

import pytest
from jira_mcp.fastmcp_server import app


@pytest.fixture
def client():
    """Create a test client for FastAPI app."""
    return TestClient(app)


class TestFastMCPServer:
    """Test case for the FastMCP server."""

    def test_root_endpoint(self, client):
        """Test the root endpoint."""
        response = client.get("/")
        assert response.status_code == 200
        data = response.json()
        assert "name" in data
        assert data["name"] == "Jira MCP Server"
        assert "endpoints" in data
        assert len(data["endpoints"]) > 0

    @patch("jira_mcp.fastmcp_server.JiraTicketManager")
    def test_get_assigned_tickets_success(self, mock_ticket_manager_class, client):
        """Test the GET /tickets/assigned endpoint with successful retrieval."""
        # Arrange
        mock_ticket_manager = MagicMock()
        mock_ticket_manager_class.return_value = mock_ticket_manager

        mock_tickets = self._create_mock_tickets(2)
        mock_ticket_manager.get_assigned_tickets.return_value = mock_tickets

        # Act
        response = client.get("/tickets/assigned")

        # Assert
        assert response.status_code == 200
        data = response.json()
        assert "tickets" in data
        assert len(data["tickets"]) == 2
        assert data["count"] == 2
        assert "markdown" in data
        mock_ticket_manager.get_assigned_tickets.assert_called_once()

    @patch("jira_mcp.fastmcp_server.JiraTicketManager")
    def test_get_assigned_tickets_no_tickets(self, mock_ticket_manager_class, client):
        """Test the GET /tickets/assigned endpoint when no tickets are found."""
        # Arrange
        mock_ticket_manager = MagicMock()
        mock_ticket_manager_class.return_value = mock_ticket_manager
        mock_ticket_manager.get_assigned_tickets.return_value = []

        # Act
        response = client.get("/tickets/assigned")

        # Assert
        assert response.status_code == 200
        data = response.json()
        assert "message" in data
        assert "No assigned tickets found" in data["message"]
        assert "tickets" in data
        assert data["tickets"] == []
        assert data["count"] == 0

    @patch("jira_mcp.fastmcp_server.JiraTicketManager")
    def test_get_recent_tickets_success(self, mock_ticket_manager_class, client):
        """Test the GET /tickets/recent endpoint with successful retrieval."""
        # Arrange
        mock_ticket_manager = MagicMock()
        mock_ticket_manager_class.return_value = mock_ticket_manager

        mock_tickets = self._create_mock_tickets(3)
        mock_ticket_manager.get_recent_tickets.return_value = mock_tickets

        # Act
        response = client.get("/tickets/recent?days=14")

        # Assert
        assert response.status_code == 200
        data = response.json()
        assert "tickets" in data
        assert len(data["tickets"]) == 3
        assert data["count"] == 3
        assert "markdown" in data
        mock_ticket_manager.get_recent_tickets.assert_called_once_with(14)

    @patch("jira_mcp.fastmcp_server.JiraTicketManager")
    def test_get_recent_tickets_default_days(self, mock_ticket_manager_class, client):
        """Test the GET /tickets/recent endpoint with default days parameter."""
        # Arrange
        mock_ticket_manager = MagicMock()
        mock_ticket_manager_class.return_value = mock_ticket_manager
        mock_ticket_manager.get_recent_tickets.return_value = []

        # Act
        response = client.get("/tickets/recent")

        # Assert
        assert response.status_code == 200
        data = response.json()
        mock_ticket_manager.get_recent_tickets.assert_called_once_with(30)
        assert "message" in data
        assert "No tickets updated in the last 30 days" in data["message"]
        assert data["count"] == 0

    @patch("jira_mcp.fastmcp_server.JiraTicketManager")
    def test_get_ticket_success(self, mock_ticket_manager_class, client):
        """Test the GET /tickets/{ticket_id} endpoint with a valid ticket ID."""
        # Arrange
        mock_ticket_manager = MagicMock()
        mock_ticket_manager_class.return_value = mock_ticket_manager

        ticket_id = "TEST-1"
        mock_ticket = self._create_mock_tickets(1)[0]
        mock_ticket_manager.get_ticket_by_id.return_value = mock_ticket

        # Act
        response = client.get(f"/tickets/{ticket_id}")

        # Assert
        assert response.status_code == 200
        data = response.json()
        assert "ticket" in data
        assert data["ticket"]["id"] == ticket_id
        assert "markdown" in data
        mock_ticket_manager.get_ticket_by_id.assert_called_once_with(ticket_id)

    @patch("jira_mcp.fastmcp_server.JiraTicketManager")
    def test_get_ticket_not_found(self, mock_ticket_manager_class, client):
        """Test the GET /tickets/{ticket_id} endpoint with a non-existent ticket ID."""
        # Arrange
        mock_ticket_manager = MagicMock()
        mock_ticket_manager_class.return_value = mock_ticket_manager
        ticket_id = "INVALID-1"
        mock_ticket_manager.get_ticket_by_id.return_value = None

        # Act
        response = client.get(f"/tickets/{ticket_id}")

        # Assert
        assert response.status_code == 404
        data = response.json()
        assert "detail" in data
        assert f"Ticket not found: {ticket_id}" in data["detail"]

    @patch("jira_mcp.fastmcp_server.JiraTicketManager")
    def test_get_ticket_comments_success(self, mock_ticket_manager_class, client):
        """Test the GET /tickets/{ticket_id}/comments endpoint with a valid ticket ID."""
        # Arrange
        mock_ticket_manager = MagicMock()
        mock_ticket_manager_class.return_value = mock_ticket_manager

        ticket_id = "TEST-1"
        mock_comments = ["Comment 1", "Comment 2", "Comment 3"]
        mock_ticket_manager.get_ticket_comments.return_value = mock_comments

        # Act
        response = client.get(f"/tickets/{ticket_id}/comments")

        # Assert
        assert response.status_code == 200
        data = response.json()
        assert "ticket_id" in data
        assert data["ticket_id"] == ticket_id
        assert "comments" in data
        assert data["comments"] == mock_comments
        assert data["count"] == 3
        assert "\n---\n".join(mock_comments) in data["markdown"]
        mock_ticket_manager.get_ticket_comments.assert_called_once_with(ticket_id)

    @patch("jira_mcp.fastmcp_server.JiraTicketManager")
    def test_get_ticket_comments_no_comments(self, mock_ticket_manager_class, client):
        """Test the GET /tickets/{ticket_id}/comments endpoint when no comments are found."""
        # Arrange
        mock_ticket_manager = MagicMock()
        mock_ticket_manager_class.return_value = mock_ticket_manager
        ticket_id = "TEST-1"
        mock_ticket_manager.get_ticket_comments.return_value = []

        # Act
        response = client.get(f"/tickets/{ticket_id}/comments")

        # Assert
        assert response.status_code == 200
        data = response.json()
        assert "message" in data
        assert "No comments found" in data["message"]
        assert "comments" in data
        assert data["comments"] == []
        assert data["count"] == 0

    def _create_mock_tickets(self, count: int):
        """Helper method to create mock ticket data."""
        tickets = []
        for i in range(1, count + 1):
            tickets.append(
                {
                    "id": f"TEST-{i}",
                    "title": f"Test ticket {i}",
                    "status": "Open",
                    "priority": "Medium",
                    "created_at": "2023-06-01T09:00:00.000+0000",
                    "updated_at": "2023-06-10T15:30:00.000+0000",
                    "url": f"https://test-jira.example.com/browse/TEST-{i}",
                }
            )
        return tickets