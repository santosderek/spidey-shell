#!/usr/bin/env python3
"""
CLI tool for testing the Jira MCP tools functionality.

This script allows for manual testing of the Jira integration
without requiring the full MCP infrastructure.

Usage:
  python -m tests.test_cli list-tickets [--filter=TYPE] [--days=DAYS]
  python -m tests.test_cli view-ticket TICKET_ID
  python -m tests.test_cli show-comments TICKET_ID
  python -m tests.test_cli status

Options:
  --filter=TYPE  Filter type: 'assigned' or 'recent' [default: assigned]
  --days=DAYS    Number of days for recent filter [default: 30]

Examples:
  python -m tests.test_cli list-tickets
  python -m tests.test_cli list-tickets --filter=recent --days=14
  python -m tests.test_cli view-ticket PROJ-123
  python -m tests.test_cli show-comments PROJ-123
  python -m tests.test_cli status
"""

import os
import sys
import json
import argparse
import logging
from dotenv import load_dotenv


# Import the MCP tools
from jira_mcp.tools import (
    mcp_list_assigned_tickets,
    mcp_list_recent_tickets,
    mcp_view_ticket,
    mcp_view_ticket_comments,
)


def setup_logging():
    """Configure logging for the CLI tool."""
    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
    )
    return logging.getLogger(__name__)


def check_environment():
    """Check for required environment variables and print status."""
    load_dotenv()

    # Check for Jira credentials
    jira_url = os.environ.get("JIRA_URL")
    jira_username = os.environ.get("JIRA_USERNAME") or os.environ.get("JIRA_API_USER")
    jira_api_token = os.environ.get("JIRA_API_TOKEN")

    print("\nJira Configuration Status:")
    print("-------------------------")
    print(f"JIRA_URL:      {'✅ Set' if jira_url else '❌ Not set'}")
    print(f"JIRA_USERNAME: {'✅ Set' if jira_username else '❌ Not set'}")
    print(f"JIRA_API_TOKEN: {'✅ Set' if jira_api_token else '❌ Not set'}")

    # If we have Cisco Jira defaults, show them
    print("\nCisco Jira Defaults:")
    print("-------------------")
    print("Domain: cisco-jira.atlassian.net")
    print("API Path: /rest/api/3/")
    print("URL: https://cisco-jira.atlassian.net")

    if not all([jira_url or True, jira_username, jira_api_token]):
        print("\n⚠️  Missing required Jira credentials!")
        print("Please set them in your environment or .env file.")
        return False

    print("\n✅ Environment variables are properly configured.")
    return True


def list_tickets(args):
    """List tickets according to the specified filter."""
    logger = setup_logging()

    logger.info(f"Fetching {args.filter} tickets...")
    
    message = {"filter": args.filter, "days": args.days}
    
    if args.filter == "assigned":
        response = mcp_list_assigned_tickets(message)
    elif args.filter == "recent":
        response = mcp_list_recent_tickets(message)
    else:
        print(f"Error: Unknown filter type '{args.filter}'")
        return 1

    if "error" in response:
        print(f"Error: {response['error']}")
        return 1

    if (
        "message" in response
        and "tickets" in response
        and len(response["tickets"]) == 0
    ):
        print(response["message"])
        return 0

    print(f"\nFound {response['count']} tickets:")
    print(response["markdown"])
    return 0


def view_ticket(args):
    """View details for a specific ticket."""
    logger = setup_logging()

    ticket_id = args.ticket_id
    logger.info(f"Fetching details for ticket {ticket_id}...")

    message = {"ticket_id": ticket_id}
    response = mcp_view_ticket(message)

    if "error" in response:
        print(f"Error: {response['error']}")
        return 1

    print(response["markdown"])
    return 0


def show_comments(args):
    """Show comments for a specific ticket."""
    logger = setup_logging()

    ticket_id = args.ticket_id
    logger.info(f"Fetching comments for ticket {ticket_id}...")

    message = {"ticket_id": ticket_id}
    response = mcp_view_ticket_comments(message)

    if "error" in response:
        print(f"Error: {response['error']}")
        return 1

    if (
        "message" in response
        and "comments" in response
        and len(response["comments"]) == 0
    ):
        print(response["message"])
        return 0

    print(f"\nFound {response['count']} comments for ticket {ticket_id}:")
    print(response["markdown"])
    return 0


def main():
    """Main entry point for the CLI tool."""
    parser = argparse.ArgumentParser(
        description="CLI tool for testing Jira MCP tools functionality"
    )
    subparsers = parser.add_subparsers(dest="command", help="Command to run")

    # List tickets command
    list_parser = subparsers.add_parser("list-tickets", help="List Jira tickets")
    list_parser.add_argument(
        "--filter",
        choices=["assigned", "recent"],
        default="assigned",
        help="Filter type (assigned or recent)",
    )
    list_parser.add_argument(
        "--days", type=int, default=30, help="Number of days for recent filter"
    )

    # View ticket command
    view_parser = subparsers.add_parser("view-ticket", help="View a specific ticket")
    view_parser.add_argument("ticket_id", help="ID of the ticket to view")

    # Show comments command
    comments_parser = subparsers.add_parser(
        "show-comments", help="Show comments for a ticket"
    )
    comments_parser.add_argument(
        "ticket_id", help="ID of the ticket to show comments for"
    )

    # Status command
    subparsers.add_parser("status", help="Check environment status")

    # Parse arguments
    args = parser.parse_args()

    if args.command == "list-tickets":
        return list_tickets(args)
    elif args.command == "view-ticket":
        return view_ticket(args)
    elif args.command == "show-comments":
        return show_comments(args)
    elif args.command == "status":
        return 0 if check_environment() else 1
    else:
        parser.print_help()
        return 1


if __name__ == "__main__":
    sys.exit(main())