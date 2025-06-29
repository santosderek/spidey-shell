"""
Tests for the JiraTicketManager client.

These tests verify that the JiraTicketManager properly
communicates with the Jira API and processes tickets.
"""

import pytest
from datetime import datetime, timedelta
from unittest.mock import patch, MagicMock

from jira_mcp.client import JiraTicketManager
from tests.utils.test_helpers import generate_test_issue


class TestJiraTicketManager:
    """Tests for the JiraTicketManager client."""
    
    def test_get_assigned_tickets(self, mock_jira_manager):
        """Test retrieving tickets assigned to the current user."""
        # When we request assigned tickets
        tickets = mock_jira_manager.get_assigned_tickets()
        
        # Then we should get back a list of tickets
        assert isinstance(tickets, list)
        assert len(tickets) > 0
        
        # And each ticket should have the required fields
        for ticket in tickets:
            assert "id" in ticket
            assert "title" in ticket
            assert "status" in ticket
            assert "url" in ticket
    
    def test_get_recent_tickets(self, mock_jira_manager):
        """Test retrieving tickets updated recently."""
        # When we request recent tickets from the past 30 days
        days = 30
        tickets = mock_jira_manager.get_recent_tickets(days)
        
        # Then we should get back a list of tickets
        assert isinstance(tickets, list)
        assert len(tickets) > 0
        
        # And each ticket should have the required fields
        for ticket in tickets:
            assert "id" in ticket
            assert "title" in ticket
            assert "status" in ticket
            assert "url" in ticket
    
    def test_get_ticket_by_id(self, mock_jira_manager):
        """Test retrieving a specific ticket by ID."""
        # When we request a specific ticket
        ticket_id = "TEST-1"
        ticket = mock_jira_manager.get_ticket_by_id(ticket_id)
        
        # Then we should get back the ticket details
        assert ticket is not None
        assert ticket["id"] == ticket_id
        assert "title" in ticket
        assert "status" in ticket
        assert "url" in ticket
    
    def test_get_nonexistent_ticket(self, mock_jira_manager):
        """Test retrieving a ticket that doesn't exist."""
        # When we request a non-existent ticket
        ticket_id = "NONEXISTENT-1"
        ticket = mock_jira_manager.get_ticket_by_id(ticket_id)
        
        # Then we should get None
        assert ticket is None
    
    def test_get_ticket_comments(self, mock_jira_manager):
        """Test retrieving comments for a ticket."""
        # When we request comments for a ticket that has comments
        ticket_id = "TEST-2"  # From our mock server, even-numbered tickets have comments
        comments = mock_jira_manager.get_ticket_comments(ticket_id)
        
        # Then we should get back a list of comments
        assert isinstance(comments, list)
        assert len(comments) > 0
        
        # And each comment should have content
        for comment in comments:
            assert comment  # Non-empty string
    
    def test_get_comments_no_comments(self, mock_jira_manager):
        """Test retrieving comments for a ticket with no comments."""
        # When we request comments for a ticket that has no comments
        ticket_id = "TEST-1"  # From our mock server, odd-numbered tickets have no comments
        comments = mock_jira_manager.get_ticket_comments(ticket_id)
        
        # Then we should get back an empty list
        assert isinstance(comments, list)
        assert len(comments) == 0
    
    def test_initialization_with_timeout(self):
        """Test that the JiraTicketManager accepts timeout configuration."""
        # Import at method level to avoid module-level imports
        from jira_mcp.client import JIRA as actual_JIRA
        
        # Create a manager with a custom timeout
        with patch('jira_mcp.client.JIRA') as mock_jira:
            mock_jira.return_value = MagicMock()
            
            manager = JiraTicketManager(
                auth=("test", "test"), 
                timeout=30,
                max_retries=5
            )
            
            # Verify that JIRA was called with the right options
            mock_jira.assert_called_once()
            options = mock_jira.call_args.kwargs['options']
            assert options['timeout'] == 30
            assert options['max_retries'] == 5