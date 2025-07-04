# Testing Guide

This document describes how to test the Jira MCP Server and provides guidelines for writing tests.

## Testing Framework

The Jira MCP Server uses [pytest](https://pytest.org/) for testing. The tests are organized in the `tests/` directory at the root of the project.

## Running the Tests

### Running All Tests

To run all tests:

```bash
# Using uv
uv run pytest

# Using the environment variable to properly resolve imports
export PYTHON_SRC=./src
uv run pytest

# With increased verbosity
export PYTHON_SRC=./src
uv run pytest -v
```

### Running Specific Tests

To run specific test files:

```bash
# Run a specific test file
uv run pytest tests/test_client.py

# Run a specific test class
uv run pytest tests/test_client.py::TestJiraTicketManager

# Run a specific test method
uv run pytest tests/test_client.py::TestJiraTicketManager::test_init_with_complete_credentials
```

### Test Coverage

To check test coverage:

```bash
# Install pytest-cov
uv pip install pytest-cov

# Run tests with coverage
uv run pytest --cov=src/jira_mcp

# Run tests with coverage and generate an HTML report
uv run pytest --cov=src/jira_mcp --cov-report=html
```

The HTML report will be available in the `htmlcov` directory.

## Test Structure

The tests are organized by module, with each module having its own test file:

- `tests/test_client.py`: Tests for `src/jira_mcp/client.py`
- `tests/test_formatters.py`: Tests for `src/jira_mcp/formatters.py`
- `tests/test_tools.py`: Tests for `src/jira_mcp/tools.py`
- `tests/test_server.py`: Tests for `src/jira_mcp/server.py`
- `tests/test_fastmcp.py`: Tests for `src/jira_mcp/fastmcp_server.py`

Within each test file, tests are organized into classes, with each class testing a specific component or functionality.

## Writing Tests

### Test Class Structure

Test classes inherit from `object` and follow the naming convention `Test{ModuleName}`:

```python
# Example test class for JiraTicketManager
class TestJiraTicketManager:
    """Test case for the JiraTicketManager class."""
    
    def test_init_with_complete_credentials(self):
        """Test initialization with complete credentials."""
        # Test code here
```

### Using Mock Objects

The tests use the `unittest.mock` module to mock external dependencies:

```python
from unittest.mock import MagicMock, patch

@patch("jira_mcp.client.JIRA")
def test_get_assigned_tickets(self, mock_jira_class):
    """Test getting assigned tickets."""
    # Arrange
    mock_jira = MagicMock()
    mock_jira_class.return_value = mock_jira
    
    mock_issues = [MagicMock() for _ in range(2)]
    mock_jira.search_issues.return_value = mock_issues
    
    manager = JiraTicketManager()
    
    # Act
    result = manager.get_assigned_tickets()
    
    # Assert
    assert len(result) == 2
    mock_jira.search_issues.assert_called_once_with("assignee = currentUser() ORDER BY updated DESC")
```

### Testing FastAPI Endpoints

FastAPI provides a `TestClient` for testing endpoints:

```python
from fastapi.testclient import TestClient

@pytest.fixture
def client():
    """Create a test client for FastAPI app."""
    return TestClient(app)

def test_root_endpoint(self, client):
    """Test the root endpoint."""
    response = client.get("/")
    assert response.status_code == 200
    data = response.json()
    assert "name" in data
    assert data["name"] == "Jira MCP Server"
```

### Helper Methods

Test classes often include helper methods to create test data:

```python
def _create_mock_tickets(self, count: int) -> List[Dict[str, Any]]:
    """Helper method to create mock ticket data."""
    tickets = []
    for i in range(1, count + 1):
        tickets.append(
            {
                "id": f"TEST-{i}",
                "title": f"Test ticket {i}",
                "status": "Open",
                "priority": "Medium",
                "created_at": "2023-06-01T09:00:00.000+0000",
                "updated_at": "2023-06-10T15:30:00.000+0000",
                "url": f"https://test-jira.example.com/browse/TEST-{i}",
            }
        )
    return tickets
```

## Test Patterns

### Arrange-Act-Assert

The tests follow the Arrange-Act-Assert pattern:

```python
def test_method(self):
    """Test description."""
    # Arrange - set up the test
    data = self._create_test_data()
    
    # Act - perform the action being tested
    result = method_under_test(data)
    
    # Assert - verify the result
    assert result == expected_result
```

### Testing Error Cases

For testing error cases, use `pytest.raises`:

```python
def test_error_case(self):
    """Test an error case."""
    with pytest.raises(ValueError, match="Expected error message"):
        method_that_should_raise_error()
```

### Parameterized Tests

For testing multiple variations of the same test, use `pytest.mark.parametrize`:

```python
@pytest.mark.parametrize(
    "input_value,expected_output",
    [
        ("value1", "output1"),
        ("value2", "output2"),
        ("value3", "output3")
    ]
)
def test_parameterized(self, input_value, expected_output):
    """Test with multiple input values."""
    result = method_under_test(input_value)
    assert result == expected_output
```

## Test Types

### Unit Tests

Unit tests test individual components in isolation:

```python
def test_format_date_just_now(self):
    """Test formatting a date that just occurred."""
    # Arrange
    date_str = datetime.now().isoformat()
    
    # Act
    result = format_date(date_str)
    
    # Assert
    assert result == "Just now"
```

### Integration Tests

Integration tests test how components work together:

```python
def test_process_message_known_command(self):
    """Test processing a message with a known command."""
    # Arrange
    message = {
        "command": "jira.list-assigned-tickets",
        "args": {}
    }
    
    # Act
    with patch("jira_mcp.tools.mcp_list_assigned_tickets") as mock_tool:
        mock_tool.return_value = {"result": "success"}
        response = process_message(message)
    
    # Assert
    assert response == {"result": "success"}
    mock_tool.assert_called_once_with({})
```

### API Tests

API tests test the API endpoints:

```python
def test_get_assigned_tickets_success(self, mock_ticket_manager_class, client):
    """Test the GET /tickets/assigned endpoint with successful retrieval."""
    # Arrange
    mock_ticket_manager = MagicMock()
    mock_ticket_manager_class.return_value = mock_ticket_manager
    
    mock_tickets = self._create_mock_tickets(2)
    mock_ticket_manager.get_assigned_tickets.return_value = mock_tickets
    
    # Act
    response = client.get("/tickets/assigned")
    
    # Assert
    assert response.status_code == 200
    data = response.json()
    assert "tickets" in data
    assert len(data["tickets"]) == 2
    assert data["count"] == 2
    assert "markdown" in data
    mock_ticket_manager.get_assigned_tickets.assert_called_once()
```

## Testing Guidelines

### 1. Test Coverage

Ensure that your tests cover:

- **Happy path**: The normal, expected use case
- **Edge cases**: Boundary conditions and unusual inputs
- **Error cases**: Expected errors and exceptions

### 2. Test Independence

Each test should be independent of other tests:

- Don't rely on state from previous tests
- Reset any global state at the start or end of each test
- Use fixtures to set up common test data

### 3. Test Speed

Tests should run quickly:

- Mock external dependencies
- Avoid unnecessary computation
- Use appropriate fixture scopes

### 4. Test Readability

Tests should be readable and maintainable:

- Use descriptive names for test methods
- Include docstrings explaining what each test does
- Follow the Arrange-Act-Assert pattern
- Use helper methods for common operations

### 5. Test Stability

Tests should be stable and reliable:

- Avoid tests that depend on external services
- Don't depend on timing or random values
- Use explicit assertions rather than boolean expressions

## Testing Commands

Here are some useful pytest commands:

```bash
# Run tests and stop on the first failure
uv run pytest -x

# Run tests with verbose output
uv run pytest -v

# Run tests and show all output
uv run pytest -s

# Run tests and show slow test times
uv run pytest --durations=10

# Run tests matching a pattern
uv run pytest -k "test_get_assigned"

# Run tests and generate JUnit XML report
uv run pytest --junitxml=report.xml
```

## Continuous Integration

When setting up continuous integration (CI) for the project, include the following steps:

1. Install dependencies
2. Run linters (e.g., flake8, mypy)
3. Run tests
4. Generate coverage report
5. Fail the build if coverage is below a specified threshold

Example CI configuration for GitHub Actions:

```yaml
name: Test

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ubuntu-latest

    steps:
    - uses: actions/checkout@v3
    
    - name: Set up Python
      uses: actions/setup-python@v4
      with:
        python-version: '3.11'
        
    - name: Install dependencies
      run: |
        python -m pip install --upgrade pip
        pip install uv
        uv pip install -e .
        uv pip install pytest pytest-cov flake8 mypy
        
    - name: Lint with flake8
      run: |
        flake8 src tests
        
    - name: Type check with mypy
      run: |
        mypy src
        
    - name: Test with pytest
      run: |
        export PYTHON_SRC=./src
        pytest --cov=src/jira_mcp --cov-report=xml
        
    - name: Upload coverage report
      uses: codecov/codecov-action@v3
      with:
        file: ./coverage.xml
```

## Mocking the Jira API

Since the Jira MCP Server relies heavily on the Jira API, most tests mock the `jira.JIRA` class:

```python
@patch("jira.JIRA")
def test_with_mocked_jira(self, mock_jira_class):
    """Test with a mocked Jira API."""
    # Arrange - set up the mock
    mock_jira = MagicMock()
    mock_jira_class.return_value = mock_jira
    
    # Configure the mock
    mock_issue = MagicMock()
    mock_issue.key = "TEST-1"
    mock_issue.fields.summary = "Test Issue"
    mock_issue.fields.status.name = "Open"
    mock_issue.fields.priority.name = "High"
    mock_issue.fields.created = "2023-06-01T09:00:00.000+0000"
    mock_issue.fields.updated = "2023-06-10T15:30:00.000+0000"
    
    mock_jira.issue.return_value = mock_issue
    
    # Create the manager with the mock
    manager = JiraTicketManager()
    
    # Act
    ticket = manager.get_ticket_by_id("TEST-1")
    
    # Assert
    assert ticket["id"] == "TEST-1"
    assert ticket["title"] == "Test Issue"
    assert ticket["status"] == "Open"
    assert ticket["priority"] == "High"
    mock_jira.issue.assert_called_once_with("TEST-1")
```

## Next Steps

- [Contributing Guidelines](./contributing.md) - How to contribute to the project
- [API Reference](./api-reference.md) - Detailed API reference