"""
Formatting utilities for Jira ticket data.

This module provides functions for formatting Jira ticket data into human-readable formats.
"""

from datetime import datetime
from typing import Dict, List, Any


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