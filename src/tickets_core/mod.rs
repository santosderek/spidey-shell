use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use std::time::{Duration, SystemTime};

// MCP Jira ticket provider implementation
pub mod jira;
pub use self::jira::JiraTicketMCPServer;

/// Represents a ticket from a ticketing system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticket {
    pub id: String,
    pub title: String,
    pub status: String,
    pub priority: Option<String>,
    pub created_at: Option<SystemTime>,
    pub updated_at: Option<SystemTime>,
    pub assigned_to: Option<String>,
    pub last_comment_at: Option<SystemTime>,
    pub url: Option<String>,
    pub description: Option<String>,
}

impl Ticket {
    /// Format creation date as a human-readable string
    pub fn formatted_created_date(&self) -> String {
        self.created_at
            .map(|time| format_time(time))
            .unwrap_or_else(|| "Unknown".to_string())
    }

    /// Format last updated date as a human-readable string
    pub fn formatted_updated_date(&self) -> String {
        self.updated_at
            .map(|time| format_time(time))
            .unwrap_or_else(|| "Unknown".to_string())
    }

    /// Format last comment date as a human-readable string
    pub fn formatted_last_comment_date(&self) -> String {
        self.last_comment_at
            .map(|time| format_time(time))
            .unwrap_or_else(|| "N/A".to_string())
    }

    /// Generate a markdown table row for this ticket
    pub fn to_markdown_row(&self) -> String {
        format!(
            "| {} | [{}]({}) | {} | {} | {} | {} |",
            self.id,
            self.title,
            self.url.as_deref().unwrap_or("#"),
            self.status,
            self.priority.as_deref().unwrap_or("N/A"),
            self.formatted_updated_date(),
            self.formatted_last_comment_date()
        )
    }
}

/// Format a system time as a human-readable string
fn format_time(time: SystemTime) -> String {
    match time.elapsed() {
        Ok(elapsed) => {
            if elapsed < Duration::from_secs(60) {
                "Just now".to_string()
            } else if elapsed < Duration::from_secs(60 * 60) {
                format!("{} minutes ago", elapsed.as_secs() / 60)
            } else if elapsed < Duration::from_secs(60 * 60 * 24) {
                format!("{} hours ago", elapsed.as_secs() / (60 * 60))
            } else if elapsed < Duration::from_secs(60 * 60 * 24 * 7) {
                format!("{} days ago", elapsed.as_secs() / (60 * 60 * 24))
            } else if elapsed < Duration::from_secs(60 * 60 * 24 * 30) {
                format!("{} weeks ago", elapsed.as_secs() / (60 * 60 * 24 * 7))
            } else {
                format!("{} months ago", elapsed.as_secs() / (60 * 60 * 24 * 30))
            }
        }
        Err(_) => "Unknown".to_string(),
    }
}

/// Error type for the ticket service
#[derive(Debug)]
pub enum TicketError {
    NetworkError(String),
    AuthError(String),
    ApiError(String),
    NotFound(String),
}

impl fmt::Display for TicketError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NetworkError(msg) => write!(f, "Network error: {}", msg),
            Self::AuthError(msg) => write!(f, "Authentication error: {}", msg),
            Self::ApiError(msg) => write!(f, "API error: {}", msg),
            Self::NotFound(msg) => write!(f, "Not found: {}", msg),
        }
    }
}

impl Error for TicketError {}

/// Trait for ticket service providers
pub trait TicketProvider {
    /// Get all tickets assigned to the current user
    fn get_assigned_tickets(&self) -> Result<Vec<Ticket>, TicketError>;

    /// Get recent tickets (created or updated within a time period)
    fn get_recent_tickets(&self, days: u32) -> Result<Vec<Ticket>, TicketError>;

    /// Get a specific ticket by ID
    fn get_ticket_by_id(&self, id: &str) -> Result<Ticket, TicketError>;

    /// Get comments for a specific ticket
    fn get_ticket_comments(&self, ticket_id: &str) -> Result<Vec<String>, TicketError>;

    /// Generate a markdown table of tickets
    fn generate_ticket_markdown_table(&self, tickets: &[Ticket]) -> String {
        let header = "| ID | Title | Status | Priority | Last Updated | Last Comment |\n";
        let divider = "|---|-------|--------|----------|-------------|-------------|\n";

        let mut table = String::new();
        table.push_str(header);
        table.push_str(divider);

        for ticket in tickets {
            table.push_str(&ticket.to_markdown_row());
            table.push('\n');
        }

        table
    }
}
