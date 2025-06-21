use crate::tickets_core::{Ticket, TicketProvider};
use crate::credentials::CredentialManager;
use crate::tickets::mock::MockTicketProvider;

/// MCP Server implementation for ticket systems
pub struct TicketMCPServer {
    provider: Box<dyn TicketProvider>,
    name: String,
}

impl TicketMCPServer {
    /// Create a new ticket MCP server
    pub fn new(name: &str, credential_manager: &CredentialManager) -> Self {
        // Use a mock provider for now
        // In a real implementation, we would look at the credentials to decide which provider to use
        let provider: Box<dyn TicketProvider> = Box::new(MockTicketProvider::new());
            
        Self {
            provider,
            name: name.to_string(),
        }
    }
    
    /// Get the name of this MCP server
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Handle a command from the user
    pub fn handle_command(&self, command: &str) -> String {
        let parts: Vec<&str> = command.trim().split_whitespace().collect();
        if parts.is_empty() {
            return self.help();
        }
        
        match parts[0] {
            "list" | "tickets" => self.list_tickets(),
            "recent" => {
                let days = if parts.len() > 1 {
                    parts[1].parse::<u32>().unwrap_or(7)
                } else {
                    7
                };
                self.list_recent_tickets(days)
            },
            "ticket" | "show" => {
                if parts.len() > 1 {
                    self.show_ticket_details(parts[1])
                } else {
                    "Error: Please provide a ticket ID".to_string()
                }
            },
            "comments" => {
                if parts.len() > 1 {
                    self.show_ticket_comments(parts[1])
                } else {
                    "Error: Please provide a ticket ID".to_string()
                }
            },
            "help" => self.help(),
            _ => format!("Unknown command: {}. Try 'help' for available commands.", parts[0]),
        }
    }
    
    /// List all assigned tickets
    fn list_tickets(&self) -> String {
        match self.provider.get_assigned_tickets() {
            Ok(tickets) => {
                if tickets.is_empty() {
                    "No assigned tickets found.".to_string()
                } else {
                    let mut result = format!("# Your Assigned Tickets ({} total)\n\n", tickets.len());
                    result.push_str(&self.provider.generate_ticket_markdown_table(&tickets));
                    result
                }
            },
            Err(e) => format!("Error retrieving tickets: {}", e),
        }
    }
    
    /// List recent tickets
    fn list_recent_tickets(&self, days: u32) -> String {
        match self.provider.get_recent_tickets(days) {
            Ok(tickets) => {
                if tickets.is_empty() {
                    format!("No tickets updated in the last {} days.", days)
                } else {
                    let mut result = format!("# Recent Tickets (Last {} days, {} total)\n\n", days, tickets.len());
                    result.push_str(&self.provider.generate_ticket_markdown_table(&tickets));
                    result
                }
            },
            Err(e) => format!("Error retrieving recent tickets: {}", e),
        }
    }
    
    /// Show details for a specific ticket
    fn show_ticket_details(&self, ticket_id: &str) -> String {
        match self.provider.get_ticket_by_id(ticket_id) {
            Ok(ticket) => {
                let mut result = format!("# Ticket {}: {}\n\n", ticket.id, ticket.title);
                result.push_str(&format!("**Status:** {}\n", ticket.status));
                if let Some(priority) = &ticket.priority {
                    result.push_str(&format!("**Priority:** {}\n", priority));
                }
                if let Some(assigned_to) = &ticket.assigned_to {
                    result.push_str(&format!("**Assigned to:** {}\n", assigned_to));
                }
                result.push_str(&format!("**Created:** {}\n", ticket.formatted_created_date()));
                result.push_str(&format!("**Updated:** {}\n", ticket.formatted_updated_date()));
                if let Some(url) = &ticket.url {
                    result.push_str(&format!("**URL:** {}\n", url));
                }
                result.push_str("\n");
                if let Some(description) = &ticket.description {
                    result.push_str("## Description\n\n");
                    result.push_str(description);
                    result.push_str("\n\n");
                }
                
                // Add comments if any
                match self.provider.get_ticket_comments(&ticket.id) {
                    Ok(comments) => {
                        if !comments.is_empty() {
                            result.push_str("## Comments\n\n");
                            for (i, comment) in comments.iter().enumerate() {
                                result.push_str(&format!("{}. {}\n", i + 1, comment));
                            }
                        }
                    },
                    Err(_) => {} // Ignore comment errors
                }
                
                result
            },
            Err(e) => format!("Error retrieving ticket {}: {}", ticket_id, e),
        }
    }
    
    /// Show comments for a specific ticket
    fn show_ticket_comments(&self, ticket_id: &str) -> String {
        match self.provider.get_ticket_comments(ticket_id) {
            Ok(comments) => {
                if comments.is_empty() {
                    format!("No comments found for ticket {}", ticket_id)
                } else {
                    let mut result = format!("# Comments for Ticket {}\n\n", ticket_id);
                    for (i, comment) in comments.iter().enumerate() {
                        result.push_str(&format!("{}. {}\n", i + 1, comment));
                    }
                    result
                }
            },
            Err(e) => format!("Error retrieving comments for ticket {}: {}", ticket_id, e),
        }
    }
    
    /// Show help information
    fn help(&self) -> String {
        r#"# Ticket MCP Server Commands

Available commands:

* `tickets` or `list` - List all assigned tickets
* `recent [days]` - List tickets updated in the last N days (default: 7)
* `ticket ID` or `show ID` - Show details for a specific ticket
* `comments ID` - Show comments for a specific ticket
* `help` - Show this help message

Examples:
```
@ticket tickets
@ticket recent 3
@ticket show TICKET-123
@ticket comments TICKET-456
```
"#.to_string()
    }
}