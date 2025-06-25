#!/usr/bin/env python3
"""
MCP tools for Jira integration.

This module provides MCP tools for interacting with Jira. It allows users to:
1. List assigned tickets
2. List recent tickets
3. View ticket details
4. View ticket comments
"""

import sys
import argparse
import logging
import json
import os
from datetime import datetime, timedelta
from typing import Dict, List, Optional, Any

# Conditionally import dotenv for development environments
try:
    from dotenv import load_dotenv

    load_dotenv()
except ImportError:
    pass  # Skip if dotenv is not available

# Conditionally import jira library
try:
    from jira import JIRA
    from jira.exceptions import JIRAError
except ImportError:
    print("Error: jira library not installed. Please run: uv pip install jira")
    sys.exit(1)

# Set up logging
logging.basicConfig(
    level=logging.INFO, format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)
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


# MCP tool utility functions
def format_date(date_str: str) -> str:
    """Format a date string in a human-readable format."""
    try:
        dt = datetime.strptime(date_str.split(".")[0], "%Y-%m-%dT%H:%M:%S")
        now = datetime.now()
        diff = now - dt

        if diff.days == 0:
            if diff.seconds < 60:
                return "Just now"
            elif diff.seconds < 3600:
                return f"{diff.seconds // 60} minutes ago"
            else:
                return f"{diff.seconds // 3600} hours ago"
        elif diff.days < 7:
            return f"{diff.days} days ago"
        elif diff.days < 30:
            return f"{diff.days // 7} weeks ago"
        else:
            return dt.strftime("%Y-%m-%d %H:%M")
    except Exception:
        return date_str


def generate_markdown_table(tickets: List[Dict[str, Any]]) -> str:
    """Generate a markdown table from a list of tickets."""
    if not tickets:
        return "No tickets found."

    header = "| ID | Title | Status | Priority | Last Updated | Last Comment |\n"
    divider = "|---|-------|--------|----------|-------------|-------------|\n"

    rows = []
    for ticket in tickets:
        # Format dates
        updated = ticket.get("updated_at", "Unknown")
        if updated != "Unknown":
            updated = format_date(updated)

        last_comment = ticket.get("last_comment_at", "N/A")
        if last_comment != "N/A":
            last_comment = format_date(last_comment)

        rows.append(
            f"| {ticket.get('id')} | [{ticket.get('title')}]({ticket.get('url', '#')}) | "
            f"{ticket.get('status')} | {ticket.get('priority', 'N/A')} | "
            f"{updated} | {last_comment} |"
        )

    return header + divider + "\n".join(rows)


def ticket_to_markdown(ticket: Dict[str, Any]) -> str:
    """Convert a ticket to a markdown representation."""
    md = [
        f"# [{ticket.get('id')}] {ticket.get('title')}",
        "",
        f"**Status:** {ticket.get('status')}",
        f"**Priority:** {ticket.get('priority', 'N/A')}",
    ]

    if ticket.get("assigned_to"):
        md.append(f"**Assigned to:** {ticket.get('assigned_to')}")

    if ticket.get("created_at"):
        created = format_date(ticket.get("created_at"))
        md.append(f"**Created:** {created}")

    if ticket.get("updated_at"):
        updated = format_date(ticket.get("updated_at"))
        md.append(f"**Last updated:** {updated}")

    md.extend(
        [
            "",
            "## Description",
            "",
            ticket.get("description", "No description provided."),
        ]
    )

    url = ticket.get("url")
    if url:
        md.extend(["", f"[View in Jira]({url})"])

    return "\n".join(md)


# MCP Tool: List Assigned Tickets
def mcp_list_assigned_tickets(args: Dict[str, Any]) -> Dict[str, Any]:
    """MCP tool to list tickets assigned to the current user."""
    manager = JiraTicketManager()
    tickets = manager.get_assigned_tickets()

    if not tickets:
        return {
            "message": "No assigned tickets found",
            "tickets": [],
        }

    markdown_table = generate_markdown_table(tickets)
    return {"tickets": tickets, "count": len(tickets), "markdown": markdown_table}


# MCP Tool: List Recent Tickets
def mcp_list_recent_tickets(args: Dict[str, Any]) -> Dict[str, Any]:
    """MCP tool to list recently updated tickets."""
    days = int(args.get("days", 30))
    manager = JiraTicketManager()
    tickets = manager.get_recent_tickets(days)

    if not tickets:
        return {
            "message": f"No tickets updated in the last {days} days",
            "tickets": [],
        }

    markdown_table = generate_markdown_table(tickets)
    return {"tickets": tickets, "count": len(tickets), "markdown": markdown_table}


# MCP Tool: View Ticket Details
def mcp_view_ticket(args: Dict[str, Any]) -> Dict[str, Any]:
    """MCP tool to view details of a specific ticket."""
    ticket_id = args.get("ticket_id")
    if not ticket_id:
        return {"error": "No ticket ID provided"}

    manager = JiraTicketManager()
    ticket = manager.get_ticket_by_id(ticket_id)

    if not ticket:
        return {"error": f"Ticket not found: {ticket_id}"}

    markdown = ticket_to_markdown(ticket)
    return {"ticket": ticket, "markdown": markdown}


# MCP Tool: View Ticket Comments
def mcp_view_ticket_comments(args: Dict[str, Any]) -> Dict[str, Any]:
    """MCP tool to view comments on a specific ticket."""
    ticket_id = args.get("ticket_id")
    if not ticket_id:
        return {"error": "No ticket ID provided"}

    manager = JiraTicketManager()
    comments = manager.get_ticket_comments(ticket_id)

    if not comments:
        return {
            "message": f"No comments found for ticket: {ticket_id}",
            "comments": [],
        }

    return {
        "ticket_id": ticket_id,
        "comments": comments,
        "count": len(comments),
        "markdown": "\n---\n".join(comments),
    }


# MCP Tool schema/registry
MCP_TOOLS = {
    "jira.list-assigned-tickets": {
        "function": mcp_list_assigned_tickets,
        "description": "List tickets assigned to the current user",
        "args": {},
    },
    "jira.list-recent-tickets": {
        "function": mcp_list_recent_tickets,
        "description": "List recently updated tickets",
        "args": {
            "days": {
                "type": "integer",
                "default": 30,
                "description": "Number of days to look back",
            }
        },
    },
    "jira.view-ticket": {
        "function": mcp_view_ticket,
        "description": "View details of a specific ticket",
        "args": {
            "ticket_id": {
                "type": "string",
                "required": True,
                "description": "Jira ticket ID",
            }
        },
    },
    "jira.view-ticket-comments": {
        "function": mcp_view_ticket_comments,
        "description": "View comments on a specific ticket",
        "args": {
            "ticket_id": {
                "type": "string",
                "required": True,
                "description": "Jira ticket ID",
            }
        },
    },
}


def main():
    """Run the Jira MCP server."""
    parser = argparse.ArgumentParser(description="Jira MCP Server")
    parser.add_argument("--debug", action="store_true", help="Enable debug logging")
    args = parser.parse_args()

    log_level = logging.DEBUG if args.debug else logging.INFO
    logging.basicConfig(
        level=log_level, format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
    )
    logger = logging.getLogger(__name__)

    logger.info("Starting Jira MCP server")

    # Process stdin messages
    for line in sys.stdin:
        try:
            # Parse incoming JSON message
            message = json.loads(line.strip())
            logger.debug(f"Received message: {message}")

            # Process the message
            command = message.get("command", "")
            response = {"error": f"Unknown command: {command}"}

            if command in MCP_TOOLS:
                tool_config = MCP_TOOLS[command]
                tool_func = tool_config["function"]
                response = tool_func(message)
            elif command == "list-commands":
                response = {
                    "commands": [
                        {
                            "name": cmd,
                            "description": cfg["description"],
                            "args": cfg["args"],
                        }
                        for cmd, cfg in MCP_TOOLS.items()
                    ]
                }

            # Send the response
            print(json.dumps(response), flush=True)
        except json.JSONDecodeError:
            logger.error("Invalid JSON received")
        except Exception as e:
            logger.error(f"Error processing message: {e}")
            print(json.dumps({"error": f"Internal server error: {str(e)}"}), flush=True)

    logger.info("Jira MCP server stopped")
    return 0


if __name__ == "__main__":
    sys.exit(main())

