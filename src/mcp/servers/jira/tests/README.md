# Jira MCP Testing Framework

This directory contains a comprehensive test suite for the Jira MCP Server, using real HTTP connections with proper isolation between tests.

## Directory Structure

```
tests/
├── conftest.py              # Global fixtures and configuration
├── __init__.py              # Package marker
├── test_basic.py            # Basic sanity tests
├── unit/                    # Unit tests
│   ├── __init__.py
│   └── test_client.py       # Tests for JiraTicketManager
├── integration/             # Integration tests
│   ├── __init__.py
│   ├── test_mcp_tools.py    # Tests for MCP tools
│   └── test_websocket.py    # Tests for WebSocket functionality
├── utils/                   # Test utilities
│   ├── __init__.py
│   └── test_helpers.py      # Helper functions for tests
└── mock_server/             # Mock server implementation
    ├── __init__.py
    ├── server.py            # FastAPI mock server
    ├── server_thread.py     # Server thread implementation
    └── fixtures.py          # Server-specific fixtures
```

## Key Components

### Mock Server

The mock server architecture provides a reliable, isolated test environment:

1. **Real HTTP Server**: Tests use an actual HTTP server running in a background thread
2. **Socket Communication**: WebSocket support for real-time communication tests
3. **Port Management**: Using random available ports to avoid conflicts
4. **Thread-based**: The server runs in a background thread to not block tests
5. **Proper Cleanup**: Clean server shutdown between test sessions

### Fixtures

The test suite uses pytest fixtures for efficient resource management:

1. **Session-scoped Server**: A single server instance shared across the test session
2. **Function-scoped Client**: Fresh JiraTicketManager instances for each test
3. **Isolated State**: Each test runs with a clean, isolated state
4. **Real HTTP Connections**: Tests use actual HTTP connections, not mocks

### Configuration

Tests can be configured via environment variables or settings:

1. **Timeout Settings**: All HTTP connections use proper timeouts
2. **Retry Logic**: Failed operations have retry capabilities
3. **Connection Pooling**: Efficient connection management
4. **Memory Management**: Tests use in-memory storage where possible

## Running Tests

Run all tests:
```bash
uv run pytest
```

Run specific test categories:
```bash
# Run only unit tests
uv run pytest tests/unit/

# Run only integration tests
uv run pytest tests/integration/

# Run a specific test file
uv run pytest tests/unit/test_client.py
```

Run with debugging output:
```bash
uv run pytest -v --log-cli-level=DEBUG
```

## Adding Tests

When adding new tests:

1. **Choose the right location**:
   - Unit tests go in `tests/unit/`
   - Integration tests go in `tests/integration/`
   - Test utilities go in `tests/utils/`

2. **Use fixtures appropriately**:
   - `mock_jira_server`: For direct access to the mock server URL
   - `mock_jira_manager`: For a configured JiraTicketManager
   - `sample_tickets`: For pre-defined test data

3. **Ensure isolation**: Make sure your tests don't affect other tests

4. **Add proper assertions**: Verify all aspects of the functionality being tested