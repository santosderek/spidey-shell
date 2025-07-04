import logging
from datetime import datetime, timedelta
from typing import Dict, List, Optional, Any, Tuple
from urllib.parse import urljoin

from jira import JIRA
from jira.exceptions import JIRAError

from jira_mcp.settings import settings

logger = logging.getLogger(__name__)


class JiraTicketManager:
    """Manages interactions with Jira API to fetch and process tickets."""

    def __init__(
        self, 
        auth: Optional[Tuple[str, str]] = None, 
        timeout: int = 10, 
        max_retries: int = 3
    ):
        """
        Initialize the Jira ticket manager with credentials.
        
        Args:
            auth: Tuple of (username, api_token) for authentication.
                 If not provided, will use credentials from settings.
            timeout: Connection timeout in seconds
            max_retries: Maximum number of connection retries
        """
        self.jira_url = settings.jira_url
        self.jira_client = None
        
        # Use provided auth or get from settings
        credentials = auth or settings.credentials_tuple
        
        if credentials:
            try:
                # Configure client options with timeouts and retries
                client_options = {
                    'timeout': timeout,
                    'max_retries': max_retries,
                    'pool_connections': 10,
                    'pool_maxsize': 10,
                }
                
                self.jira_client = JIRA(
                    server=self.jira_url,
                    basic_auth=credentials,
                    options=client_options,
                )
                logger.info(f"Connected to Jira at {self.jira_url}")
            except Exception as e:
                logger.error(f"Failed to connect to Jira: {e}")
        else:
            logger.warning("No credentials provided. Some operations will be unavailable.")

    def get_assigned_tickets(self) -> List[Dict[str, Any]]:
        """Get all tickets assigned to the current user."""
        if not self.jira_client:  # FIXME: Make this an error type
            return []

        try:
            jql_query = "assignee = currentUser() ORDER BY updated DESC"
            issues = self.jira_client.search_issues(jql_query)
            return [self._convert_to_ticket(issue) for issue in issues]
        except JIRAError as e:
            logger.error(f"Error fetching assigned tickets: {e}")
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
            logger.error(f"Error fetching recent tickets: {e}")
            return []

    def get_ticket_by_id(self, ticket_id: str) -> Optional[Dict[str, Any]]:
        """Get a specific ticket by ID."""
        if not self.jira_client:
            return None

        try:
            issue = self.jira_client.issue(ticket_id)
            return self._convert_to_ticket(issue)
        except JIRAError as e:
            logger.error(f"Error fetching ticket {ticket_id}: {e}")
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
        except JIRAError:
            logger.exception(f"Error fetching comments for ticket {ticket_id}")
            return []
        except Exception:
            logger.exception("Unexpected error processing comments.")
            return []

    def _convert_to_ticket(self, issue) -> Dict[str, Any]:
        """Convert a Jira issue to a normalized ticket format."""
        try:
            # Extract basic fields
            ticket = {
                "id": issue.key,
                "title": issue.fields.summary,
                "status": issue.fields.status.name,
                "url": f"{self.jira_client.server_url}/browse/{issue.key}",
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
        except Exception:
            return {
                "id": issue.key,
                "title": "Error processing ticket",
                "status": "Unknown",
                "url": urljoin(self.jira_client.server_url, f"/browse/{issue.key}"),
            }
