use crate::credentials::CredentialManager;
use crate::tickets_core::{Ticket, TicketError, TicketProvider};
use serde_json::{json, Value};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn};

/// Jira ticket provider implementation using MCP
pub struct JiraTicketMCPServer {
    name: String,
    credential_manager: CredentialManager,
}

impl JiraTicketMCPServer {
    /// Create a new Jira MCP ticket server
    pub fn new(name: &str, credential_manager: CredentialManager) -> Self {
        Self {
            name: name.to_string(),
            credential_manager,
        }
    }

    /// Get the name of this server
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Convert a MCP ticket response to a Ticket object
    fn convert_mcp_ticket(&self, ticket_data: &Value) -> Option<Ticket> {
        let id = ticket_data.get("id")?.as_str()?.to_string();
        let title = ticket_data.get("title")?.as_str()?.to_string();
        let status = ticket_data.get("status")?.as_str()?.to_string();

        // Optional fields
        let priority = ticket_data
            .get("priority")
            .and_then(|p| p.as_str())
            .map(|s| s.to_string());
        let url = ticket_data
            .get("url")
            .and_then(|u| u.as_str())
            .map(|s| s.to_string());
        let description = ticket_data
            .get("description")
            .and_then(|d| d.as_str())
            .map(|s| s.to_string());
        let assigned_to = ticket_data
            .get("assigned_to")
            .and_then(|a| a.as_str())
            .map(|s| s.to_string());

        // Parse timestamps if present
        let created_at =
            self.parse_timestamp(ticket_data.get("created_at").and_then(|t| t.as_str()));
        let updated_at =
            self.parse_timestamp(ticket_data.get("updated_at").and_then(|t| t.as_str()));
        let last_comment_at =
            self.parse_timestamp(ticket_data.get("last_comment_at").and_then(|t| t.as_str()));

        Some(Ticket {
            id,
            title,
            status,
            priority,
            created_at,
            updated_at,
            assigned_to,
            last_comment_at,
            url,
            description,
        })
    }

    /// Parse an ISO timestamp into SystemTime
    fn parse_timestamp(&self, timestamp: Option<&str>) -> Option<SystemTime> {
        match timestamp {
            Some(ts) => {
                // Parse ISO8601 format like "2021-09-01T12:34:56.789Z"
                // For simplicity, we'll just extract the seconds since epoch
                if let Ok(datetime) = chrono::DateTime::parse_from_rfc3339(ts) {
                    let secs = datetime.timestamp();
                    if secs > 0 {
                        return Some(UNIX_EPOCH + std::time::Duration::from_secs(secs as u64));
                    }
                }
                None
            }
            None => None,
        }
    }

    /// Prepare MCP server by setting credentials
    pub fn prepare(&self) -> Result<(), TicketError> {
        // Set up environment variables for the MCP server
        let jira_url = self.credential_manager.get("JIRA_URL");
        let jira_username = self.credential_manager.get("JIRA_USERNAME");
        let jira_api_token = self.credential_manager.get("JIRA_API_TOKEN");

        if jira_url.is_none() || jira_username.is_none() || jira_api_token.is_none() {
            warn!("Missing Jira credentials for MCP server");
            return Err(TicketError::AuthError(
                "Missing Jira credentials. Please set JIRA_URL, JIRA_USERNAME, and JIRA_API_TOKEN"
                    .to_string(),
            ));
        }

        Ok(())
    }

    /// Helper to send a command to the MCP server
    async fn send_mcp_command(&self, command: &str, params: Value) -> Result<Value, TicketError> {
        let mut message = json!({
            "command": command
        });

        // Merge the params into the message
        if let Some(obj) = message.as_object_mut() {
            if let Some(params_obj) = params.as_object() {
                for (k, v) in params_obj {
                    obj.insert(k.clone(), v.clone());
                }
            }
        }

        // TODO: Send the message to the MCP server and get a response
        // For now, we'll simulate with a placeholder
        // This would be replaced with actual MCP server communication
        info!("Sending MCP command {} to Jira server", command);
        debug!("MCP message: {:?}", message);

        // This is a placeholder - in a real implementation, we would:
        // 1. Get the MCPServerManager
        // 2. Use manager.send_to_python_process(self.name(), &message.to_string())
        // 3. Parse the response

        Err(TicketError::ApiError(
            "MCP communication not fully implemented".to_string(),
        ))
    }
}

impl TicketProvider for JiraTicketMCPServer {
    fn get_assigned_tickets(&self) -> Result<Vec<Ticket>, TicketError> {
        // This would be implemented with async/await in a real application
        // For now we'll use a blocking approach for simplicity

        // Prepare the credentials
        self.prepare()?;

        match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                // We're in an async context, use block_on
                match handle
                    .block_on(self.send_mcp_command("list-tickets", json!({"filter": "assigned"})))
                {
                    Ok(response) => {
                        let tickets = response
                            .get("tickets")
                            .and_then(|t| t.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|ticket_data| self.convert_mcp_ticket(ticket_data))
                                    .collect()
                            })
                            .unwrap_or_default();

                        Ok(tickets)
                    }
                    Err(e) => Err(e),
                }
            }
            Err(_) => {
                // We're not in an async context, create a new runtime
                let rt = tokio::runtime::Runtime::new().map_err(|e| {
                    TicketError::ApiError(format!("Failed to create Tokio runtime: {}", e))
                })?;

                match rt
                    .block_on(self.send_mcp_command("list-tickets", json!({"filter": "assigned"})))
                {
                    Ok(response) => {
                        let tickets = response
                            .get("tickets")
                            .and_then(|t| t.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|ticket_data| self.convert_mcp_ticket(ticket_data))
                                    .collect()
                            })
                            .unwrap_or_default();

                        Ok(tickets)
                    }
                    Err(e) => Err(e),
                }
            }
        }
    }

    fn get_recent_tickets(&self, days: u32) -> Result<Vec<Ticket>, TicketError> {
        // This would be implemented with async/await in a real application
        // For now we'll use a blocking approach for simplicity

        // Prepare the credentials
        self.prepare()?;

        match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                // We're in an async context, use block_on
                match handle.block_on(
                    self.send_mcp_command(
                        "list-tickets",
                        json!({"filter": "recent", "days": days}),
                    ),
                ) {
                    Ok(response) => {
                        let tickets = response
                            .get("tickets")
                            .and_then(|t| t.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|ticket_data| self.convert_mcp_ticket(ticket_data))
                                    .collect()
                            })
                            .unwrap_or_default();

                        Ok(tickets)
                    }
                    Err(e) => Err(e),
                }
            }
            Err(_) => {
                // We're not in an async context, create a new runtime
                let rt = tokio::runtime::Runtime::new().map_err(|e| {
                    TicketError::ApiError(format!("Failed to create Tokio runtime: {}", e))
                })?;

                match rt.block_on(
                    self.send_mcp_command(
                        "list-tickets",
                        json!({"filter": "recent", "days": days}),
                    ),
                ) {
                    Ok(response) => {
                        let tickets = response
                            .get("tickets")
                            .and_then(|t| t.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|ticket_data| self.convert_mcp_ticket(ticket_data))
                                    .collect()
                            })
                            .unwrap_or_default();

                        Ok(tickets)
                    }
                    Err(e) => Err(e),
                }
            }
        }
    }

    fn get_ticket_by_id(&self, id: &str) -> Result<Ticket, TicketError> {
        // Prepare the credentials
        self.prepare()?;

        match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                // We're in an async context, use block_on
                match handle
                    .block_on(self.send_mcp_command("view-ticket", json!({"ticket_id": id})))
                {
                    Ok(response) => {
                        let ticket_data = response.get("ticket").ok_or_else(|| {
                            TicketError::ApiError("No ticket data in response".to_string())
                        })?;

                        self.convert_mcp_ticket(ticket_data).ok_or_else(|| {
                            TicketError::ApiError("Failed to convert ticket data".to_string())
                        })
                    }
                    Err(e) => Err(e),
                }
            }
            Err(_) => {
                // We're not in an async context, create a new runtime
                let rt = tokio::runtime::Runtime::new().map_err(|e| {
                    TicketError::ApiError(format!("Failed to create Tokio runtime: {}", e))
                })?;

                match rt.block_on(self.send_mcp_command("view-ticket", json!({"ticket_id": id}))) {
                    Ok(response) => {
                        let ticket_data = response.get("ticket").ok_or_else(|| {
                            TicketError::ApiError("No ticket data in response".to_string())
                        })?;

                        self.convert_mcp_ticket(ticket_data).ok_or_else(|| {
                            TicketError::ApiError("Failed to convert ticket data".to_string())
                        })
                    }
                    Err(e) => Err(e),
                }
            }
        }
    }

    fn get_ticket_comments(&self, ticket_id: &str) -> Result<Vec<String>, TicketError> {
        // Prepare the credentials
        self.prepare()?;

        match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                // We're in an async context, use block_on
                match handle.block_on(
                    self.send_mcp_command("show-ticket-comments", json!({"ticket_id": ticket_id})),
                ) {
                    Ok(response) => {
                        let comments = response
                            .get("comments")
                            .and_then(|c| c.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|comment| comment.as_str().map(|s| s.to_string()))
                                    .collect()
                            })
                            .unwrap_or_default();

                        Ok(comments)
                    }
                    Err(e) => Err(e),
                }
            }
            Err(_) => {
                // We're not in an async context, create a new runtime
                let rt = tokio::runtime::Runtime::new().map_err(|e| {
                    TicketError::ApiError(format!("Failed to create Tokio runtime: {}", e))
                })?;

                match rt.block_on(
                    self.send_mcp_command("show-ticket-comments", json!({"ticket_id": ticket_id})),
                ) {
                    Ok(response) => {
                        let comments = response
                            .get("comments")
                            .and_then(|c| c.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|comment| comment.as_str().map(|s| s.to_string()))
                                    .collect()
                            })
                            .unwrap_or_default();

                        Ok(comments)
                    }
                    Err(e) => Err(e),
                }
            }
        }
    }
}
