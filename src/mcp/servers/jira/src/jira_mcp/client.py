"""
Jira client module for interacting with the Jira API.

This module provides a JiraTicketManager class for fetching and processing Jira tickets.
"""

import os
import logging
from datetime import datetime, timedelta
from typing import Dict, List, Optional, Any

try:
    from jira import JIRA
    from jira.exceptions import JIRAError
except ImportError:
    # This will be handled when the manager is instantiated
    pass

logger = logging.getLogger(__name__)


class JiraTicketManager:
    """Manages interactions with Jira API to fetch and process tickets."""

    # Cisco Jira defaults
    jira_cloud_scheme = "https"
    jira_cloud_domain = "cisco-jira.atlassian.net"
    jira_cloud_api_path = "/rest/api/3/"

    def __init__(self):
        """Initialize the Jira ticket manager with credentials from environment."""
        # Use environment variables or defaults
        self.jira_url = (
            os.getenv("JIRA_URL")
            or f"{self.jira_cloud_scheme}://{self.jira_cloud_domain}"
        )
        self.jira_username = os.getenv("JIRA_API_USER")
        self.jira_api_token = os.getenv("JIRA_API_TOKEN")
        self.jira_client = None
        self.logger = logging.getLogger(__name__)

        # If JIRA_URL is not set, construct it from defaults
        if not self.jira_url and self.jira_cloud_domain:
            self.jira_url = f"{self.jira_cloud_scheme}://{self.jira_cloud_domain}"
            self.logger.info(f"Using default Jira URL: {self.jira_url}")

        if not all([self.jira_url, self.jira_username, self.jira_api_token]):
            missing = []
            if not self.jira_url:
                missing.append("JIRA_URL")
            if not self.jira_username:
                missing.append("JIRA_USERNAME/JIRA_API_USER")
            if not self.jira_api_token:
                missing.append("JIRA_API_TOKEN")
            self.logger.warning(f"Missing Jira credentials: {', '.join(missing)}")
        else:
            try:
                self.jira_client = JIRA(
                    server=self.jira_url,
                    basic_auth=(self.jira_username, self.jira_api_token),
                )
                self.logger.info("Successfully connected to Jira")
            except Exception as e:
                self.logger.error(f"Failed to connect to Jira: {e}")

    def get_assigned_tickets(self) -> List[Dict[str, Any]]:
        """Get all tickets assigned to the current user."""
        if not self.jira_client:
            return []

        try:
            jql_query = "assignee = currentUser() ORDER BY updated DESC"
            issues = self.jira_client.search_issues(jql_query)
            return [self._convert_to_ticket(issue) for issue in issues]
        except JIRAError as e:
            self.logger.error(f"Error fetching assigned tickets: {e}")
            return []

    def get_recent_tickets(self, days: int) -> List[Dict[str, Any]]:
        """Get tickets created or updated within the specified number of days."""
        if not self.jira_client:
            return []

        try:
            date_str = (datetime.now() - timedelta(days=days)).strftime("%Y-%m-%d")
            jql_query = f"updated >= {date_str} ORDER BY updated DESC"
            issues = self.jira_client.search_issues(jql_query)
            return [self._convert_to_ticket(issue) for issue in issues]
        except JIRAError as e:
            self.logger.error(f"Error fetching recent tickets: {e}")
            return []

    def get_ticket_by_id(self, ticket_id: str) -> Optional[Dict[str, Any]]:
        """Get a specific ticket by ID."""
        if not self.jira_client:
            return None

        try:
            issue = self.jira_client.issue(ticket_id)
            return self._convert_to_ticket(issue)
        except JIRAError as e:
            self.logger.error(f"Error fetching ticket {ticket_id}: {e}")
            return None

    def get_ticket_comments(self, ticket_id: str) -> List[str]:
        """Get comments for a specific ticket."""
        if not self.jira_client:
            return []

        try:
            issue = self.jira_client.issue(ticket_id)
            comments = []

            for comment in issue.fields.comment.comments:
                author = comment.author.displayName
                created = datetime.strptime(
                    comment.created.split(".")[0], "%Y-%m-%dT%H:%M:%S"
                ).strftime("%Y-%m-%d %H:%M:%S")

                comments.append(f"**{author}** on {created}:\n\n{comment.body}\n")

            return comments
        except JIRAError as e:
            self.logger.error(f"Error fetching comments for ticket {ticket_id}: {e}")
            return []
        except Exception as e:
            self.logger.error(f"Unexpected error processing comments: {e}")
            return []

    def _convert_to_ticket(self, issue) -> Dict[str, Any]:
        """Convert a Jira issue to a normalized ticket format."""
        try:
            # Extract basic fields
            ticket = {
                "id": issue.key,
                "title": issue.fields.summary,
                "status": issue.fields.status.name,
                "url": f"{self.jira_url}/browse/{issue.key}",
            }

            # Extract optional fields
            if hasattr(issue.fields, "priority") and issue.fields.priority:
                ticket["priority"] = issue.fields.priority.name

            if hasattr(issue.fields, "assignee") and issue.fields.assignee:
                ticket["assigned_to"] = issue.fields.assignee.displayName

            if hasattr(issue.fields, "description") and issue.fields.description:
                ticket["description"] = issue.fields.description

            # Handle timestamps
            if hasattr(issue.fields, "created") and issue.fields.created:
                ticket["created_at"] = issue.fields.created

            if hasattr(issue.fields, "updated") and issue.fields.updated:
                ticket["updated_at"] = issue.fields.updated

            if hasattr(issue.fields, "comment") and issue.fields.comment.comments:
                # Get the most recent comment time
                latest_comment = max(
                    issue.fields.comment.comments, key=lambda c: c.updated
                )
                ticket["last_comment_at"] = latest_comment.updated

            return ticket
        except Exception as e:
            self.logger.error(f"Error converting issue {issue.key}: {e}")
            return {
                "id": issue.key,
                "title": "Error processing ticket",
                "status": "Unknown",
                "url": f"{self.jira_url}/browse/{issue.key}",
            }