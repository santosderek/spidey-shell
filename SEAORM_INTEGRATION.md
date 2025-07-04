# SeaORM Integration for Spidey Shell

This document describes the SeaORM integration for the Spidey Shell application, which provides a more robust ORM-based database interface compared to the current direct rusqlite implementation.

## Overview

The SeaORM integration provides the following benefits:
- Type-safe database queries
- Asynchronous database operations
- Better separation of concerns with repository pattern
- More maintainable and extensible code structure

## Important: Dependency Conflict Issue

Due to fundamental dependency conflicts between rusqlite and SeaORM (both depend on different versions of libsqlite3-sys), **you cannot compile and run both implementations in the same binary or workspace**. 

There are two ways to work with this code:

1. **Fork-based approach (recommended):**
   - Create two separate forks of the project
   - In one fork, use only rusqlite (the current implementation)
   - In another fork, replace rusqlite with SeaORM

2. **Manual switching:**
   - Comment out either the rusqlite-related code or the SeaORM-related code
   - Adjust Cargo.toml to include only the necessary dependencies
   - This approach requires careful changes to multiple files

## Code Structure

The SeaORM integration code is organized as follows:

### Entity Models
- `src/persistence/entities/conversation.rs` - Entity model for conversations
- `src/persistence/entities/generation.rs` - Entity model for generations (message history)
- `src/persistence/entities/prelude.rs` - Re-exports for entity models
- `src/persistence/entities/mod.rs` - Module exports

### Database Setup
- `src/persistence/database.rs` - Database connection manager and migration logic

### Repositories
- `src/persistence/repositories/conversation_repository.rs` - Conversation data operations
- `src/persistence/repositories/generation_repository.rs` - Generation (message) data operations
- `src/persistence/repositories/mod.rs` - Module exports

### Main Store Interface
- `src/persistence/seaorm_store.rs` - Main interface for the application to interact with the database

### Application Integration
- `src/app/sea_orm_state.rs` - SeaORM version of the AppState
- `src/app/sea_orm_input.rs` - SeaORM version of input handling
- `src/app/sea_orm_events.rs` - SeaORM version of event handling
- `src/bin/sea_orm_main.rs` - Entry point for the SeaORM version of the app

### Migration Strategy

If you want to implement this in a real application, follow these steps:

1. **First, create a data migration tool:**
   - Create a separate tool that reads from the rusqlite database
   - Export the data in a format that SeaORM can import (JSON, CSV, etc.)

2. **Then, implement the SeaORM version:**
   - Create a new project or branch without rusqlite
   - Add all the SeaORM code
   - Import the data from the export file

3. **Testing:**
   - Test both implementations separately
   - Ensure data integrity after migration

## How to Run Tests

To run the original rusqlite-based tests:

```bash
# Temporarily comment out or remove SeaORM-related code, then:
cargo test
```

To test the SeaORM implementation:

```bash
# Temporarily comment out or remove rusqlite-related code, then:
cargo test
```

## Database Schema

Both implementations use the same underlying schema:

1. `conversations`/`conversation` - Stores conversation metadata
   - `id` - Primary key
   - `title` - Conversation title
   - `created_at` - Creation timestamp
   - `mcp_servers` - JSON string containing enabled MCP servers

2. `generations`/`message` - Stores message history
   - `id` - Primary key
   - `conversation_id` - Foreign key to conversations
   - `role` - Message role (user/assistant)
   - `content` - Message content
   - `created_at` - Creation timestamp
   - `mcp_server` - Optional MCP server name

3. `mcp_activation` - Tracks MCP server activations
   - `id` - Primary key
   - `conversation_id` - Foreign key to conversations
   - `mcp_server` - MCP server name
   - `is_active` - Whether the server is active
   - `activated_at` - Activation timestamp

## Implementation Details

### Database Schema

The SeaORM integration uses the following tables:

1. `conversations` - Stores conversation metadata
   - `id` - Primary key
   - `title` - Conversation title
   - `created_at` - Creation timestamp
   - `mcp_servers` - JSON string containing enabled MCP servers

2. `generations` - Stores message history
   - `id` - Primary key
   - `conversation_id` - Foreign key to conversations
   - `role` - Message role (user/assistant)
   - `content` - Message content
   - `created_at` - Creation timestamp
   - `mcp_server` - Optional MCP server name

3. `mcp_activation` - Tracks MCP server activations
   - `id` - Primary key
   - `conversation_id` - Foreign key to conversations
   - `mcp_server` - MCP server name
   - `is_active` - Whether the server is active
   - `activated_at` - Activation timestamp

### Notes on Integration

The SeaORM implementation maintains compatibility with the existing app structure but uses async functions throughout. Key differences:

1. Database operations are all asynchronous
2. Repositories provide a cleaner interface for data operations
3. Entity models provide compile-time type safety
4. Database connections are managed more efficiently