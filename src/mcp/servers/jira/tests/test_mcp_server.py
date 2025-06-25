"""
Tests for the Jira MCP tools.
"""

from unittest.mock import MagicMock, patch
from typing import Dict, List, Any
from datetime import datetime, timedelta

# Add parent directory to path for imports
import os
import sys

sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))
from src.jira_mcp.main import (
    JiraTicketManager,
    generate_markdown_table,
    ticket_to_markdown,
    format_date,
)


class TestJiraMCPTools:
    """Test case for the Jira MCP tools."""

    def setup_method(self):
        """Set up the test environment for each test method."""
        # No need to patch JiraTicketManager anymore - we'll use direct testing

    def test_generate_markdown_table(self):
        """Test markdown table generation from tickets."""
        # Arrange
        tickets = self._create_mock_tickets(2)

        # Act
        table = generate_markdown_table(tickets)

        # Assert
        assert "| ID | Title | Status | Priority | Last Updated | Last Comment |" in table
        assert "|---|-------|--------|----------|-------------|-------------|" in table
        assert "| TEST-1 | [Test ticket 1](https://test-jira.example.com/browse/TEST-1) | Open" in table
        assert "| TEST-2 | [Test ticket 2](https://test-jira.example.com/browse/TEST-2) | Open" in table

    def test_generate_markdown_table_no_tickets(self):
        """Test markdown table generation with no tickets."""
        # Act
        table = generate_markdown_table([])

        # Assert
        assert table == "No tickets found."

    def test_ticket_to_markdown(self):
        """Test markdown conversion of a ticket."""
        # Arrange
        ticket = {
            "id": "TEST-1",
            "title": "Test ticket 1",
            "status": "Open",
            "priority": "Medium",
            "assigned_to": "Test User",
            "created_at": "2023-06-01T09:00:00.000+0000",
            "updated_at": "2023-06-10T15:30:00.000+0000",
            "description": "Test description",
            "url": "https://test-jira.example.com/browse/TEST-1",
        }

        # Act (with monkey-patched format_date for predictable output)
        with patch("src.jira_mcp.main.format_date", return_value="2023-06-10 15:30"):
            markdown = ticket_to_markdown(ticket)

        # Assert
        assert "# [TEST-1] Test ticket 1" in markdown
        assert "**Status:** Open" in markdown
        assert "**Priority:** Medium" in markdown
        assert "**Assigned to:** Test User" in markdown
        assert "**Created:** 2023-06-10 15:30" in markdown
        assert "**Last updated:** 2023-06-10 15:30" in markdown
        assert "## Description" in markdown
        assert "Test description" in markdown
        assert "[View in Jira](https://test-jira.example.com/browse/TEST-1)" in markdown

    def test_format_date_just_now(self):
        """Test date formatting for very recent dates."""
        # Arrange
        now = datetime.now().strftime("%Y-%m-%dT%H:%M:%S.000+0000")

        # Act
        result = format_date(now)

        # Assert
        assert result == "Just now"

    def test_format_date_minutes(self):
        """Test date formatting for dates within the hour."""
        # Arrange
        # 30 minutes ago
        dt = datetime.now() - timedelta(minutes=30)
        date_str = dt.strftime("%Y-%m-%dT%H:%M:%S.000+0000")

        # Act
        result = format_date(date_str)

        # Assert
        assert "minutes ago" in result

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