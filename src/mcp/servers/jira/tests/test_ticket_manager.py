"""
Tests for the JiraTicketManager class.
"""

import os
from unittest.mock import MagicMock, patch
from datetime import datetime, timedelta

from jira import JIRAError
from jira.resources import Issue

# Import the module directly from our package
# Add parent directory to path for imports

from jira_mcp.client import JiraTicketManager


class TestJiraTicketManager:
    """Test case for the JiraTicketManager class."""

    def setup_method(self):
        """Set up test environment before each test method."""
        # Save original environment variables
        self.orig_env = {
            "JIRA_URL": os.environ.get("JIRA_URL"),
            "JIRA_USERNAME": os.environ.get("JIRA_USERNAME"),
            "JIRA_API_USER": os.environ.get("JIRA_API_USER"),
            "JIRA_API_TOKEN": os.environ.get("JIRA_API_TOKEN"),
        }

        # Set test environment variables
        os.environ["JIRA_URL"] = "https://test-jira.example.com"
        os.environ["JIRA_API_USER"] = "test_user"
        os.environ["JIRA_API_TOKEN"] = "test_token"

    def teardown_method(self):
        """Clean up test environment after each test method."""
        # Restore original environment variables
        for key, value in self.orig_env.items():
            if value is None:
                if key in os.environ:
                    del os.environ[key]
            else:
                os.environ[key] = value

    @patch("jira_mcp.client.JIRA")
    def test_init_with_complete_credentials(self, mock_jira):
        """Test initialization with all credentials provided."""
        # Arrange
        mock_jira.return_value = MagicMock()

        # Act
        manager = JiraTicketManager()

        # Assert
        assert manager.jira_url == "https://test-jira.example.com"
        assert manager.jira_username == "test_user"
        assert manager.jira_api_token == "test_token"
        mock_jira.assert_called_once_with(
            server="https://test-jira.example.com",
            basic_auth=("test_user", "test_token"),
        )
        assert manager.jira_client is not None

    @patch("jira_mcp.client.JIRA")
    def test_init_with_cisco_defaults(self, mock_jira):
        """Test initialization with Cisco defaults."""
        # Arrange
        mock_jira.return_value = MagicMock()

        # Remove JIRA_URL to test defaults
        if "JIRA_URL" in os.environ:
            del os.environ["JIRA_URL"]

        # Act
        manager = JiraTicketManager()

        # Assert
        assert manager.jira_url == "https://cisco-jira.atlassian.net"
        mock_jira.assert_called_once_with(
            server="https://cisco-jira.atlassian.net",
            basic_auth=("test_user", "test_token"),
        )

    @patch("jira_mcp.client.JIRA")
    def test_init_with_api_user(self, mock_jira):
        """Test initialization with JIRA_API_USER instead of JIRA_USERNAME."""
        # Arrange
        mock_jira.return_value = MagicMock()

        # Use a different API user
        os.environ["JIRA_API_USER"] = "api_user"

        # Act
        manager = JiraTicketManager()

        # Assert
        assert manager.jira_username == "api_user"
        mock_jira.assert_called_once_with(
            server="https://test-jira.example.com",
            basic_auth=("api_user", "test_token"),
        )

    @patch("jira_mcp.client.JIRA")
    def test_init_connection_error(self, mock_jira):
        """Test handling of Jira connection error."""
        # Arrange
        mock_jira.side_effect = Exception("Connection error")

        # Act
        manager = JiraTicketManager()

        # Assert
        assert manager.jira_client is None

    def test_get_assigned_tickets_no_client(self):
        """Test get_assigned_tickets when no client is available."""
        # Arrange
        with patch("jira_mcp.client.JIRA") as mock_jira:
            mock_jira.side_effect = Exception("Connection error")
            manager = JiraTicketManager()

        # Act
        tickets = manager.get_assigned_tickets()

        # Assert
        assert tickets == []

    @patch("jira_mcp.client.JIRA")
    def test_get_assigned_tickets_success(self, mock_jira):
        """Test successful retrieval of assigned tickets."""
        # Arrange
        mock_client = MagicMock()
        mock_jira.return_value = mock_client

        # Mock issues
        issue1 = self._create_mock_issue("TEST-1", "Test Issue 1", "Open")
        issue2 = self._create_mock_issue("TEST-2", "Test Issue 2", "In Progress")
        mock_client.search_issues.return_value = [issue1, issue2]

        manager = JiraTicketManager()

        # Act
        tickets = manager.get_assigned_tickets()

        # Assert
        assert len(tickets) == 2
        assert tickets[0]["id"] == "TEST-1"
        assert tickets[1]["id"] == "TEST-2"
        mock_client.search_issues.assert_called_once_with(
            "assignee = currentUser() ORDER BY updated DESC"
        )

    @patch("jira_mcp.client.JIRA")
    def test_get_assigned_tickets_error(self, mock_jira):
        """Test error handling in get_assigned_tickets."""
        # Arrange
        mock_client = MagicMock()
        mock_jira.return_value = mock_client
        mock_client.search_issues.side_effect = JIRAError("Error fetching tickets")

        manager = JiraTicketManager()

        # Act
        tickets = manager.get_assigned_tickets()

        # Assert
        assert tickets == []

    @patch("jira_mcp.client.JIRA")
    def test_get_recent_tickets_success(self, mock_jira):
        """Test successful retrieval of recent tickets."""
        # Arrange
        mock_client = MagicMock()
        mock_jira.return_value = mock_client

        # Mock issues
        issue1 = self._create_mock_issue("TEST-3", "Recent Issue 1", "Open")
        issue2 = self._create_mock_issue("TEST-4", "Recent Issue 2", "Resolved")
        mock_client.search_issues.return_value = [issue1, issue2]

        manager = JiraTicketManager()

        # Act
        tickets = manager.get_recent_tickets(7)

        # Assert
        assert len(tickets) == 2
        assert tickets[0]["id"] == "TEST-3"
        assert tickets[1]["id"] == "TEST-4"

        # Calculate expected date string for 7 days ago
        date_str = (datetime.now() - timedelta(days=7)).strftime("%Y-%m-%d")
        mock_client.search_issues.assert_called_once_with(
            f"updated >= {date_str} ORDER BY updated DESC"
        )

    @patch("jira_mcp.client.JIRA")
    def test_get_ticket_by_id_success(self, mock_jira):
        """Test successful retrieval of a ticket by ID."""
        # Arrange
        mock_client = MagicMock()
        mock_jira.return_value = mock_client

        # Mock issue
        ticket_id = "TEST-5"
        issue = self._create_mock_issue(ticket_id, "Single Issue", "Open")
        mock_client.issue.return_value = issue

        manager = JiraTicketManager()

        # Act
        ticket = manager.get_ticket_by_id(ticket_id)

        # Assert
        assert ticket is not None
        assert ticket["id"] == ticket_id
        assert ticket["title"] == "Single Issue"
        mock_client.issue.assert_called_once_with(ticket_id)

    @patch("jira_mcp.client.JIRA")
    def test_get_ticket_by_id_error(self, mock_jira):
        """Test error handling in get_ticket_by_id."""
        # Arrange
        mock_client = MagicMock()
        mock_jira.return_value = mock_client
        mock_client.issue.side_effect = JIRAError("Issue not found")

        manager = JiraTicketManager()

        # Act
        ticket = manager.get_ticket_by_id("INVALID-1")

        # Assert
        assert ticket is None

    @patch("jira_mcp.client.JIRA")
    def test_get_ticket_comments_success(self, mock_jira):
        """Test successful retrieval of ticket comments."""
        # Arrange
        mock_client = MagicMock()
        mock_jira.return_value = mock_client

        # Mock issue with comments
        ticket_id = "TEST-6"
        issue = self._create_mock_issue(ticket_id, "Issue with Comments", "Open")

        # Create mock comments
        comment1 = MagicMock()
        comment1.author.displayName = "John Doe"
        comment1.created = "2023-06-15T10:30:45.000+0000"
        comment1.body = "This is comment 1"

        comment2 = MagicMock()
        comment2.author.displayName = "Jane Smith"
        comment2.created = "2023-06-16T14:20:15.000+0000"
        comment2.body = "This is comment 2"

        # Attach comments to issue
        issue.fields.comment.comments = [comment1, comment2]
        mock_client.issue.return_value = issue

        manager = JiraTicketManager()

        # Act
        comments = manager.get_ticket_comments(ticket_id)

        # Assert
        assert len(comments) == 2
        assert "John Doe" in comments[0]
        assert "2023-06-15 10:30:45" in comments[0]
        assert "This is comment 1" in comments[0]
        assert "Jane Smith" in comments[1]
        mock_client.issue.assert_called_once_with(ticket_id)

    def _create_mock_issue(self, key, summary, status_name):
        """Helper method to create a mock Jira issue."""
        issue = MagicMock(spec=Issue)
        issue.key = key

        # Create fields as a MagicMock
        fields = MagicMock()
        issue.fields = fields

        # Set up fields
        fields.summary = summary
        fields.status = MagicMock()
        fields.status.name = status_name
        fields.priority = MagicMock()
        fields.priority.name = "Medium"
        fields.created = "2023-06-01T09:00:00.000+0000"
        fields.updated = "2023-06-10T15:30:00.000+0000"
        fields.assignee = MagicMock()
        fields.assignee.displayName = "Test User"
        fields.description = f"Description for {key}"

        # Set up comments
        comment = MagicMock()
        comment.author = MagicMock()
        comment.author.displayName = "Commenter"
        comment.created = "2023-06-05T10:15:00.000+0000"
        comment.updated = "2023-06-05T10:15:00.000+0000"
        comment.body = "Test comment"

        fields.comment = MagicMock()
        fields.comment.comments = [comment]

        return issue
