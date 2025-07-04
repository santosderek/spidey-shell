"""
Integration tests for MCP tools.

These tests verify that the MCP tools properly integrate
with the JiraTicketManager and can retrieve and format tickets.
"""

import pytest
from unittest.mock import patch

from jira_mcp.server.tools import list_tickets, view_ticket, show_ticket_comments
from jira_mcp.client import JiraTicketManager


class TestMCPTools:
    """Integration tests for MCP tools with real server."""

    def test_list_tickets_assigned(self, mock_jira_manager):
        """Test listing assigned tickets through MCP tools."""
        # Given a configured JiraTicketManager from fixture
        with patch('jira_mcp.server.tools.get_ticket_manager', return_value=mock_jira_manager):
            # When we call the list_tickets tool with 'assigned' filter
            result = list_tickets({"filter": "assigned"})
            
            # Then we should get a proper response
            assert "tickets" in result
            assert "count" in result
            assert "markdown" in result
            assert result["count"] > 0
            assert len(result["tickets"]) == result["count"]
            assert "Found" in result["message"]
    
    def test_list_tickets_recent(self, mock_jira_manager):
        """Test listing recent tickets through MCP tools."""
        # Given a configured JiraTicketManager from fixture
        with patch('jira_mcp.server.tools.get_ticket_manager', return_value=mock_jira_manager):
            # When we call the list_tickets tool with 'recent' filter
            result = list_tickets({"filter": "recent", "days": 7})
            
            # Then we should get a proper response
            assert "tickets" in result
            assert "count" in result
            assert "markdown" in result
            assert result["count"] > 0
            assert len(result["tickets"]) == result["count"]
            assert "Found" in result["message"]
    
    def test_view_ticket(self, mock_jira_manager):
        """Test viewing a specific ticket through MCP tools."""
        # Given a configured JiraTicketManager from fixture
        with patch('jira_mcp.server.tools.get_ticket_manager', return_value=mock_jira_manager):
            # When we call the view_ticket tool with a valid ticket ID
            result = view_ticket({"ticket_id": "TEST-1"})
            
            # Then we should get a proper response
            assert "ticket" in result
            assert "markdown" in result
            assert result["ticket"]["id"] == "TEST-1"
    
    def test_view_ticket_nonexistent(self, mock_jira_manager):
        """Test viewing a nonexistent ticket through MCP tools."""
        # Given a configured JiraTicketManager from fixture
        with patch('jira_mcp.server.tools.get_ticket_manager', return_value=mock_jira_manager):
            # When we call the view_ticket tool with an invalid ticket ID
            # Then we should get a ValueError
            with pytest.raises(ValueError, match="Ticket not found"):
                view_ticket({"ticket_id": "NONEXISTENT-1"})
    
    def test_show_ticket_comments(self, mock_jira_manager):
        """Test viewing comments for a ticket through MCP tools."""
        # Given a configured JiraTicketManager from fixture
        with patch('jira_mcp.server.tools.get_ticket_manager', return_value=mock_jira_manager):
            # When we call the show_ticket_comments tool with a valid ticket ID that has comments
            result = show_ticket_comments({"ticket_id": "TEST-2"})
            
            # Then we should get a proper response
            assert "comments" in result
            assert "count" in result
            assert "markdown" in result
            assert result["count"] > 0
            assert len(result["comments"]) == result["count"]
    
    def test_show_ticket_comments_no_comments(self, mock_jira_manager):
        """Test viewing comments for a ticket with no comments through MCP tools."""
        # Given a configured JiraTicketManager from fixture
        with patch('jira_mcp.server.tools.get_ticket_manager', return_value=mock_jira_manager):
            # When we call the show_ticket_comments tool with a valid ticket ID that has no comments
            result = show_ticket_comments({"ticket_id": "TEST-1"})
            
            # Then we should get a proper response with no comments
            assert "comments" in result
            assert "count" in result
            assert "markdown" in result
            assert result["count"] == 0
            assert len(result["comments"]) == 0
            assert "No comments found" in result["message"]