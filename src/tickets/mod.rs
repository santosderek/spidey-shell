pub mod mock;
pub mod mcp_server;

pub use self::mcp_server::TicketMCPServer;

// Re-export the ticket types and traits
pub use crate::tickets_core::{Ticket, TicketError, TicketProvider};