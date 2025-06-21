use std::time::{Duration, SystemTime};
use crate::tickets_core::{Ticket, TicketError, TicketProvider};

/// A mock ticket provider for testing or when real credentials are not available
pub struct MockTicketProvider;

impl MockTicketProvider {
    /// Create a new mock ticket provider
    pub fn new() -> Self {
        Self
    }
    
    /// Generate mock tickets
    fn generate_mock_tickets(&self) -> Vec<Ticket> {
        let now = SystemTime::now();
        vec![
            Ticket {
                id: "TICKET-123".to_string(),
                title: "Fix authentication bug in login page".to_string(),
                status: "In Progress".to_string(),
                priority: Some("High".to_string()),
                created_at: Some(now.checked_sub(Duration::from_secs(60 * 60 * 24 * 5)).unwrap()),
                updated_at: Some(now.checked_sub(Duration::from_secs(60 * 60 * 2)).unwrap()),
                assigned_to: Some("Current User".to_string()),
                last_comment_at: Some(now.checked_sub(Duration::from_secs(60 * 60 * 3)).unwrap()),
                url: Some("https://example.com/tickets/TICKET-123".to_string()),
                description: Some("Users are unable to log in using SSO in certain browsers.".to_string()),
            },
            Ticket {
                id: "TICKET-456".to_string(),
                title: "Implement new dashboard features".to_string(),
                status: "Open".to_string(),
                priority: Some("Medium".to_string()),
                created_at: Some(now.checked_sub(Duration::from_secs(60 * 60 * 24 * 2)).unwrap()),
                updated_at: Some(now.checked_sub(Duration::from_secs(60 * 60 * 24 * 1)).unwrap()),
                assigned_to: Some("Current User".to_string()),
                last_comment_at: Some(now.checked_sub(Duration::from_secs(60 * 60 * 24 * 1)).unwrap()),
                url: Some("https://example.com/tickets/TICKET-456".to_string()),
                description: Some("Add visualizations for user activity metrics.".to_string()),
            },
            Ticket {
                id: "TICKET-789".to_string(),
                title: "Database performance optimization".to_string(),
                status: "Code Review".to_string(),
                priority: Some("High".to_string()),
                created_at: Some(now.checked_sub(Duration::from_secs(60 * 60 * 24 * 10)).unwrap()),
                updated_at: Some(now.checked_sub(Duration::from_secs(60 * 30)).unwrap()),
                assigned_to: Some("Current User".to_string()),
                last_comment_at: Some(now.checked_sub(Duration::from_secs(60 * 30)).unwrap()),
                url: Some("https://example.com/tickets/TICKET-789".to_string()),
                description: Some("Optimize database queries to improve dashboard loading time.".to_string()),
            },
            Ticket {
                id: "TICKET-101".to_string(),
                title: "Update documentation for API endpoints".to_string(),
                status: "Ready for QA".to_string(),
                priority: Some("Low".to_string()),
                created_at: Some(now.checked_sub(Duration::from_secs(60 * 60 * 24 * 3)).unwrap()),
                updated_at: Some(now.checked_sub(Duration::from_secs(60 * 60 * 12)).unwrap()),
                assigned_to: Some("Current User".to_string()),
                last_comment_at: None,
                url: Some("https://example.com/tickets/TICKET-101".to_string()),
                description: Some("Update API documentation to reflect recent changes.".to_string()),
            },
            Ticket {
                id: "TICKET-202".to_string(),
                title: "Fix mobile responsive design issues".to_string(),
                status: "Open".to_string(),
                priority: Some("Medium".to_string()),
                created_at: Some(now.checked_sub(Duration::from_secs(60 * 60 * 1)).unwrap()),
                updated_at: Some(now.checked_sub(Duration::from_secs(60 * 30)).unwrap()),
                assigned_to: Some("Current User".to_string()),
                last_comment_at: Some(now.checked_sub(Duration::from_secs(60 * 30)).unwrap()),
                url: Some("https://example.com/tickets/TICKET-202".to_string()),
                description: Some("Address UI issues on mobile devices.".to_string()),
            },
        ]
    }
    
    /// Generate mock comments for a ticket
    fn generate_mock_comments(&self, ticket_id: &str) -> Vec<String> {
        match ticket_id {
            "TICKET-123" => vec![
                "Initial report: Users on Chrome can't log in using SSO".to_string(),
                "Investigated: Found issue in the OAuth callback handling".to_string(),
                "Working on a fix now, should be ready today".to_string(),
            ],
            "TICKET-456" => vec![
                "Requirements finalized for the dashboard features".to_string(),
                "Starting implementation of user activity charts".to_string(),
            ],
            "TICKET-789" => vec![
                "Identified slow queries in the dashboard loading".to_string(),
                "Added indexes to improve performance".to_string(),
                "Implemented query caching".to_string(),
                "Ready for code review - performance improved by 60%".to_string(),
            ],
            "TICKET-101" => vec![],
            "TICKET-202" => vec![
                "Found responsive design issues on iPhone and Android devices".to_string(),
                "Working on CSS fixes for navigation menu".to_string(),
            ],
            _ => vec!["No comments available".to_string()],
        }
    }
}

impl TicketProvider for MockTicketProvider {
    fn get_assigned_tickets(&self) -> Result<Vec<Ticket>, TicketError> {
        Ok(self.generate_mock_tickets())
    }
    
    fn get_recent_tickets(&self, days: u32) -> Result<Vec<Ticket>, TicketError> {
        let now = SystemTime::now();
        let threshold = now.checked_sub(Duration::from_secs(days as u64 * 24 * 60 * 60))
            .unwrap_or(now);
            
        let all_tickets = self.generate_mock_tickets();
        Ok(all_tickets.into_iter()
            .filter(|t| t.updated_at.unwrap_or(now) > threshold)
            .collect())
    }
    
    fn get_ticket_by_id(&self, id: &str) -> Result<Ticket, TicketError> {
        let all_tickets = self.generate_mock_tickets();
        all_tickets.into_iter()
            .find(|t| t.id == id)
            .ok_or_else(|| TicketError::NotFound(format!("Ticket with ID {} not found", id)))
    }
    
    fn get_ticket_comments(&self, ticket_id: &str) -> Result<Vec<String>, TicketError> {
        Ok(self.generate_mock_comments(ticket_id))
    }
}