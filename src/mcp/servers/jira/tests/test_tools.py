"""
Tests for the Jira MCP tools.
"""

import pytest
from unittest.mock import MagicMock, patch
from typing import Dict, List, Any

from jira_mcp.tools import (
    mcp_list_assigned_tickets,
    mcp_list_recent_tickets,
    mcp_view_ticket,
    mcp_view_ticket_comments,
)


class TestJiraMCPTools:
    """Test case for the Jira MCP tools."""

    @patch("jira_mcp.tools.JiraTicketManager")
    def test_mcp_list_assigned_tickets_success(self, mock_ticket_manager_class):
        """Test the jira.list-assigned-tickets tool with successful retrieval."""
        # Arrange
        mock_ticket_manager = MagicMock()
        mock_ticket_manager_class.return_value = mock_ticket_manager

        mock_tickets = self._create_mock_tickets(2)
        mock_ticket_manager.get_assigned_tickets.return_value = mock_tickets

        # Act
        response = mcp_list_assigned_tickets({})

        # Assert
        assert "tickets" in response
        assert response["tickets"] == mock_tickets
        assert response["count"] == 2
        assert "markdown" in response
        mock_ticket_manager.get_assigned_tickets.assert_called_once()

    @patch("jira_mcp.tools.JiraTicketManager")
    def test_mcp_list_assigned_tickets_no_tickets(self, mock_ticket_manager_class):
        """Test the jira.list-assigned-tickets tool when no tickets are found."""
        # Arrange
        mock_ticket_manager = MagicMock()
        mock_ticket_manager_class.return_value = mock_ticket_manager
        mock_ticket_manager.get_assigned_tickets.return_value = []

        # Act
        response = mcp_list_assigned_tickets({})

        # Assert
        assert "message" in response
        assert "No assigned tickets" in response["message"]
        assert "tickets" in response
        assert response["tickets"] == []

    @patch("jira_mcp.tools.JiraTicketManager")
    def test_mcp_list_recent_tickets_success(self, mock_ticket_manager_class):
        """Test the jira.list-recent-tickets tool with successful retrieval."""
        # Arrange
        mock_ticket_manager = MagicMock()
        mock_ticket_manager_class.return_value = mock_ticket_manager

        mock_tickets = self._create_mock_tickets(3)
        mock_ticket_manager.get_recent_tickets.return_value = mock_tickets

        # Act
        response = mcp_list_recent_tickets({"days": 14})

        # Assert
        assert "tickets" in response
        assert response["tickets"] == mock_tickets
        assert response["count"] == 3
        assert "markdown" in response
        mock_ticket_manager.get_recent_tickets.assert_called_once_with(14)

    @patch("jira_mcp.tools.JiraTicketManager")
    def test_mcp_list_recent_tickets_default_days(self, mock_ticket_manager_class):
        """Test the jira.list-recent-tickets tool with default days parameter."""
        # Arrange
        mock_ticket_manager = MagicMock()
        mock_ticket_manager_class.return_value = mock_ticket_manager
        mock_ticket_manager.get_recent_tickets.return_value = []

        # Act
        response = mcp_list_recent_tickets({})

        # Assert
        mock_ticket_manager.get_recent_tickets.assert_called_once_with(30)
        assert "message" in response
        assert "No tickets updated in the last 30 days" in response["message"]

    @patch("jira_mcp.tools.JiraTicketManager")
    def test_mcp_view_ticket_success(self, mock_ticket_manager_class):
        """Test the jira.view-ticket tool with a valid ticket ID."""
        # Arrange
        mock_ticket_manager = MagicMock()
        mock_ticket_manager_class.return_value = mock_ticket_manager

        ticket_id = "TEST-1"
        mock_ticket = self._create_mock_tickets(1)[0]
        mock_ticket_manager.get_ticket_by_id.return_value = mock_ticket

        # Act
        response = mcp_view_ticket({"ticket_id": ticket_id})

        # Assert
        assert "ticket" in response
        assert response["ticket"] == mock_ticket
        assert "markdown" in response
        mock_ticket_manager.get_ticket_by_id.assert_called_once_with(ticket_id)

    @patch("jira_mcp.tools.JiraTicketManager")
    def test_mcp_view_ticket_no_id(self, mock_ticket_manager_class):
        """Test the jira.view-ticket tool without a ticket ID."""
        # Act & Assert
        with pytest.raises(ValueError, match="No ticket ID provided"):
            mcp_view_ticket({})

    @patch("jira_mcp.tools.JiraTicketManager")
    def test_mcp_view_ticket_not_found(self, mock_ticket_manager_class):
        """Test the jira.view-ticket tool with a non-existent ticket ID."""
        # Arrange
        mock_ticket_manager = MagicMock()
        mock_ticket_manager_class.return_value = mock_ticket_manager
        ticket_id = "INVALID-1"
        mock_ticket_manager.get_ticket_by_id.return_value = None

        # Act & Assert
        with pytest.raises(ValueError, match=f"Ticket not found: {ticket_id}"):
            mcp_view_ticket({"ticket_id": ticket_id})

    @patch("jira_mcp.tools.JiraTicketManager")
    def test_mcp_view_ticket_comments_success(self, mock_ticket_manager_class):
        """Test the jira.view-ticket-comments tool with a valid ticket ID."""
        # Arrange
        mock_ticket_manager = MagicMock()
        mock_ticket_manager_class.return_value = mock_ticket_manager

        ticket_id = "TEST-1"
        mock_comments = ["Comment 1", "Comment 2", "Comment 3"]
        mock_ticket_manager.get_ticket_comments.return_value = mock_comments

        # Act
        response = mcp_view_ticket_comments({"ticket_id": ticket_id})

        # Assert
        assert "ticket_id" in response
        assert response["ticket_id"] == ticket_id
        assert "comments" in response
        assert response["comments"] == mock_comments
        assert response["count"] == 3
        assert "\n---\n".join(mock_comments) in response["markdown"]
        mock_ticket_manager.get_ticket_comments.assert_called_once_with(ticket_id)

    @patch("jira_mcp.tools.JiraTicketManager")
    def test_mcp_view_ticket_comments_no_id(self, mock_ticket_manager_class):
        """Test the jira.view-ticket-comments tool without a ticket ID."""
        # Act & Assert
        with pytest.raises(ValueError, match="No ticket ID provided"):
            mcp_view_ticket_comments({})

    @patch("jira_mcp.tools.JiraTicketManager")
    def test_mcp_view_ticket_comments_no_comments(self, mock_ticket_manager_class):
        """Test the jira.view-ticket-comments tool when no comments are found."""
        # Arrange
        mock_ticket_manager = MagicMock()
        mock_ticket_manager_class.return_value = mock_ticket_manager
        ticket_id = "TEST-1"
        mock_ticket_manager.get_ticket_comments.return_value = []

        # Act
        response = mcp_view_ticket_comments({"ticket_id": ticket_id})

        # Assert
        assert "message" in response
        assert "No comments found" in response["message"]
        assert "comments" in response
        assert response["comments"] == []

    def _create_mock_tickets(self, count: int) -> List[Dict[str, Any]]:
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