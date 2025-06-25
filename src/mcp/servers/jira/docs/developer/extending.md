# Extending the Server

This guide provides instructions on how to extend the Jira MCP Server with new functionality.

## Adding New MCP Tools

To add a new MCP tool:

1. Add a new function in `src/jira_mcp/tools.py`:

```python
def mcp_my_new_tool(args: Dict[str, Any]) -> Dict[str, Any]:
    """MCP tool to perform a new function."""
    # Parse arguments
    required_arg = args.get("required_arg")
    if not required_arg:
        raise ValueError("No required_arg provided")
        
    # Implement your functionality
    result = perform_your_operation(required_arg)
    
    # Return results in a consistent format
    return {
        "result": result,
        "markdown": format_result_as_markdown(result)
    }
```

2. Register the tool in the `MCP_TOOLS` dictionary in `src/jira_mcp/tools.py`:

```python
MCP_TOOLS = {
    # ... existing tools ...
    "jira.my-new-tool": {
        "function": mcp_my_new_tool,
        "description": "Performs a new function",
        "args": {
            "required_arg": {
                "type": "string",
                "required": True,
                "description": "A required argument"
            },
            "optional_arg": {
                "type": "integer",
                "default": 10,
                "description": "An optional argument"
            }
        }
    }
}
```

3. Add a FastMCP endpoint in `src/jira_mcp/fastmcp_server.py`:

```python
# Add a Pydantic model for your request
class MyNewToolRequest(BaseModel):
    required_arg: str = Field(..., description="A required argument")
    optional_arg: int = Field(10, description="An optional argument")

# Add a Pydantic model for your response  
class MyNewToolResponse(BaseModel):
    result: Any
    markdown: str

# Add the FastMCP tool decorator
@mcp.tool()
def my_new_tool(args: Dict[str, Any]) -> Dict[str, Any]:
    """
    Performs a new function.
    
    Args:
        args: Dictionary containing parameters:
            - required_arg: A required argument (required)
            - optional_arg: An optional argument (default: 10)
            
    Returns a result with markdown formatting.
    """
    return mcp_my_new_tool(args)

# Add a FastAPI endpoint
@app.get("/my-new-endpoint", response_model=MyNewToolResponse)
async def get_my_new_endpoint(required_arg: str, optional_arg: int = 10):
    """Performs a new function."""
    try:
        return my_new_tool({"required_arg": required_arg, "optional_arg": optional_arg})
    except ValueError as e:
        logger.warning(f"Error: {e}")
        raise HTTPException(status_code=400, detail=str(e))
    except Exception as e:
        logger.error(f"Error performing operation: {e}")
        raise HTTPException(status_code=500, detail=str(e))
```

4. Add tests for your new tool in `tests/test_tools.py`:

```python
@patch("jira_mcp.tools.perform_your_operation")
def test_mcp_my_new_tool_success(self, mock_perform):
    """Test the jira.my-new-tool with valid arguments."""
    # Arrange
    mock_perform.return_value = "success"
    
    # Act
    response = mcp_my_new_tool({"required_arg": "test"})
    
    # Assert
    assert "result" in response
    assert response["result"] == "success"
    assert "markdown" in response
    mock_perform.assert_called_once_with("test")

def test_mcp_my_new_tool_no_arg(self):
    """Test the jira.my-new-tool without required argument."""
    # Act & Assert
    with pytest.raises(ValueError, match="No required_arg provided"):
        mcp_my_new_tool({})
```

5. Add tests for the FastAPI endpoint in `tests/test_fastmcp.py`:

```python
@patch("jira_mcp.fastmcp_server.my_new_tool")
def test_get_my_new_endpoint_success(self, mock_tool, client):
    """Test the GET /my-new-endpoint endpoint with valid arguments."""
    # Arrange
    mock_tool.return_value = {"result": "success", "markdown": "Success!"}
    
    # Act
    response = client.get("/my-new-endpoint?required_arg=test&optional_arg=20")
    
    # Assert
    assert response.status_code == 200
    data = response.json()
    assert "result" in data
    assert data["result"] == "success"
    assert "markdown" in data
    mock_tool.assert_called_once_with({"required_arg": "test", "optional_arg": 20})

@patch("jira_mcp.fastmcp_server.my_new_tool")
def test_get_my_new_endpoint_error(self, mock_tool, client):
    """Test the GET /my-new-endpoint endpoint with an error."""
    # Arrange
    mock_tool.side_effect = ValueError("Test error")
    
    # Act
    response = client.get("/my-new-endpoint?required_arg=test")
    
    # Assert
    assert response.status_code == 400
    data = response.json()
    assert "detail" in data
    assert "Test error" in data["detail"]
```

6. Update the API documentation in both `docs/user-guide/api-reference.md` and `docs/developer/api-reference.md`.

## Extending the Jira Client

To add new functionality to the Jira client:

1. Add a new method to `JiraTicketManager` in `src/jira_mcp/client.py`:

```python
def get_custom_data(self, parameter: str) -> List[Dict[str, Any]]:
    """Get custom data from Jira."""
    if not self.jira_client:
        return []
        
    try:
        # Implement your custom Jira API logic
        custom_jql = f"project = {parameter} ORDER BY created DESC"
        issues = self.jira_client.search_issues(custom_jql)
        return [self._convert_to_ticket(issue) for issue in issues]
    except JIRAError as e:
        self.logger.error(f"Error fetching custom data: {e}")
        return []
```

2. Update the tests in `tests/test_client.py`:

```python
@patch("jira.JIRA")
def test_get_custom_data(self, mock_jira_class):
    """Test getting custom data from Jira."""
    # Arrange
    mock_jira = MagicMock()
    mock_jira_class.return_value = mock_jira
    
    mock_issues = [MagicMock() for _ in range(2)]
    mock_jira.search_issues.return_value = mock_issues
    
    manager = JiraTicketManager()
    
    # Act
    result = manager.get_custom_data("TEST")
    
    # Assert
    assert len(result) == 2
    mock_jira.search_issues.assert_called_once_with("project = TEST ORDER BY created DESC")
```

## Adding Custom Formatters

To add custom formatting for your data:

1. Add a new function to `src/jira_mcp/formatters.py`:

```python
def format_custom_data(data: List[Dict[str, Any]]) -> str:
    """Format custom data as markdown."""
    if not data:
        return "No data found."
        
    # Implement your custom formatting logic
    result = "# Custom Data\n\n"
    for item in data:
        result += f"- **{item['name']}**: {item['value']}\n"
    
    return result
```

2. Update the tests in `tests/test_formatters.py`:

```python
def test_format_custom_data(self):
    """Test formatting custom data."""
    # Arrange
    data = [
        {"name": "Item 1", "value": "Value 1"},
        {"name": "Item 2", "value": "Value 2"}
    ]
    
    # Act
    result = format_custom_data(data)
    
    # Assert
    assert "# Custom Data" in result
    assert "**Item 1**: Value 1" in result
    assert "**Item 2**: Value 2" in result
```

## Adding New API Endpoints

To add new REST API endpoints:

1. Add a new endpoint to `src/jira_mcp/fastmcp_server.py`:

```python
@app.get("/custom-endpoint/{parameter}")
async def get_custom_endpoint(parameter: str, query_param: Optional[str] = None):
    """Custom endpoint with path and query parameters."""
    try:
        # Implement your custom endpoint logic
        manager = JiraTicketManager()
        data = manager.get_custom_data(parameter)
        
        if query_param:
            data = [item for item in data if query_param in item["name"]]
            
        markdown = format_custom_data(data)
        
        return {"data": data, "count": len(data), "markdown": markdown}
    except Exception as e:
        logger.error(f"Error processing custom endpoint: {e}")
        raise HTTPException(status_code=500, detail=str(e))
```

2. Update the tests in `tests/test_fastmcp.py`:

```python
@patch("jira_mcp.fastmcp_server.JiraTicketManager")
def test_get_custom_endpoint(self, mock_manager_class, client):
    """Test the GET /custom-endpoint/{parameter} endpoint."""
    # Arrange
    mock_manager = MagicMock()
    mock_manager_class.return_value = mock_manager
    
    mock_data = [
        {"name": "Item 1", "value": "Value 1"},
        {"name": "Item 2", "value": "Value 2"}
    ]
    mock_manager.get_custom_data.return_value = mock_data
    
    # Act
    response = client.get("/custom-endpoint/TEST?query_param=Item")
    
    # Assert
    assert response.status_code == 200
    data = response.json()
    assert "data" in data
    assert "count" in data
    assert "markdown" in data
    assert data["count"] == 2
    mock_manager.get_custom_data.assert_called_once_with("TEST")
```

## Modifying Existing Functionality

To modify existing functionality:

1. Make your changes to the relevant module.
2. Update the tests to reflect your changes.
3. Update the documentation to reflect your changes.

## Best Practices

When extending the server, follow these best practices:

1. **Maintain Consistency**: Follow the existing code structure and patterns.
2. **Add Tests**: Write comprehensive tests for your new functionality.
3. **Update Documentation**: Keep the documentation in sync with your changes.
4. **Error Handling**: Handle errors gracefully and provide meaningful error messages.
5. **Type Hints**: Use Python type hints to improve code readability and enable static type checking.
6. **Docstrings**: Add docstrings to all functions and classes.
7. **Validation**: Validate input parameters and handle edge cases.
8. **Logging**: Use the existing logging infrastructure to log important events and errors.

## Advanced Extensions

### Adding Support for New Authentication Methods

If you need to support additional authentication methods for Jira:

1. Modify `JiraTicketManager.__init__` in `src/jira_mcp/client.py` to handle the new authentication method.
2. Update the relevant configuration documentation.

### Adding Support for Additional Jira APIs

To add support for additional Jira APIs:

1. Add new methods to `JiraTicketManager` in `src/jira_mcp/client.py`.
2. Create new MCP tools that use these methods.
3. Add new FastAPI endpoints that expose these tools.

### Adding Support for WebSockets

If you want to add WebSocket support for real-time updates:

1. Install the required dependencies:
   ```bash
   uv pip install websockets
   ```

2. Add WebSocket endpoint in `src/jira_mcp/fastmcp_server.py`:
   ```python
   @app.websocket("/ws")
   async def websocket_endpoint(websocket: WebSocket):
       await websocket.accept()
       try:
           while True:
               data = await websocket.receive_json()
               response = process_message(data)
               await websocket.send_json(response)
       except WebSocketDisconnect:
           logger.info("WebSocket disconnected")
   ```

3. Add appropriate documentation and tests.

## Example: Adding a New Tool

Here's a complete example of adding a new tool to search for Jira tickets by keyword:

### 1. Add a new method to `JiraTicketManager`

```python
# In src/jira_mcp/client.py

def search_tickets_by_keyword(self, keyword: str, max_results: int = 10) -> List[Dict[str, Any]]:
    """Search for tickets by keyword."""
    if not self.jira_client:
        return []
        
    try:
        jql_query = f'text ~ "{keyword}" ORDER BY updated DESC'
        issues = self.jira_client.search_issues(jql_query, maxResults=max_results)
        return [self._convert_to_ticket(issue) for issue in issues]
    except JIRAError as e:
        self.logger.error(f"Error searching tickets by keyword: {e}")
        return []
```

### 2. Add a new MCP tool

```python
# In src/jira_mcp/tools.py

def mcp_search_tickets(args: Dict[str, Any]) -> Dict[str, Any]:
    """MCP tool to search for tickets by keyword."""
    keyword = args.get("keyword")
    if not keyword:
        raise ValueError("No keyword provided")
        
    max_results = int(args.get("max_results", 10))
    
    manager = JiraTicketManager()
    tickets = manager.search_tickets_by_keyword(keyword, max_results)
    
    if not tickets:
        return {
            "message": f"No tickets found matching keyword: {keyword}",
            "tickets": [],
            "count": 0,
            "markdown": "No tickets found."
        }
    
    markdown_table = generate_markdown_table(tickets)
    return {"tickets": tickets, "count": len(tickets), "markdown": markdown_table}

# Update the MCP_TOOLS dictionary
MCP_TOOLS = {
    # ... existing tools ...
    "jira.search-tickets": {
        "function": mcp_search_tickets,
        "description": "Search for tickets by keyword",
        "args": {
            "keyword": {
                "type": "string",
                "required": True,
                "description": "Keyword to search for"
            },
            "max_results": {
                "type": "integer",
                "default": 10,
                "description": "Maximum number of results to return"
            }
        }
    }
}
```

### 3. Add a FastMCP endpoint

```python
# In src/jira_mcp/fastmcp_server.py

# Add a Pydantic model for the request
class SearchTicketsRequest(BaseModel):
    keyword: str = Field(..., description="Keyword to search for")
    max_results: int = Field(10, description="Maximum number of results to return")

@mcp.tool()
def search_tickets(args: Dict[str, Any]) -> Dict[str, Any]:
    """
    Search for tickets by keyword.
    
    Args:
        args: Dictionary containing parameters:
            - keyword: Keyword to search for (required)
            - max_results: Maximum number of results to return (default: 10)
            
    Returns a list of tickets matching the keyword, formatted as a markdown table.
    """
    return mcp_search_tickets(args)

@app.get("/tickets/search", response_model=ListTicketsResponse)
async def search_tickets_endpoint(keyword: str, max_results: int = 10):
    """Search for tickets by keyword."""
    try:
        return search_tickets({"keyword": keyword, "max_results": max_results})
    except ValueError as e:
        logger.warning(f"Search error: {e}")
        raise HTTPException(status_code=400, detail=str(e))
    except Exception as e:
        logger.error(f"Error searching tickets: {e}")
        raise HTTPException(status_code=500, detail=str(e))
```

### 4. Add tests

```python
# In tests/test_tools.py

@patch("jira_mcp.tools.JiraTicketManager")
def test_mcp_search_tickets_success(self, mock_ticket_manager_class):
    """Test the jira.search-tickets tool with successful search."""
    # Arrange
    mock_ticket_manager = MagicMock()
    mock_ticket_manager_class.return_value = mock_ticket_manager
    
    mock_tickets = self._create_mock_tickets(2)
    mock_ticket_manager.search_tickets_by_keyword.return_value = mock_tickets
    
    # Act
    response = mcp_search_tickets({"keyword": "test", "max_results": 5})
    
    # Assert
    assert "tickets" in response
    assert response["tickets"] == mock_tickets
    assert response["count"] == 2
    assert "markdown" in response
    mock_ticket_manager.search_tickets_by_keyword.assert_called_once_with("test", 5)

@patch("jira_mcp.tools.JiraTicketManager")
def test_mcp_search_tickets_no_keyword(self, mock_ticket_manager_class):
    """Test the jira.search-tickets tool without a keyword."""
    # Act & Assert
    with pytest.raises(ValueError, match="No keyword provided"):
        mcp_search_tickets({})

@patch("jira_mcp.tools.JiraTicketManager")
def test_mcp_search_tickets_no_results(self, mock_ticket_manager_class):
    """Test the jira.search-tickets tool with no results."""
    # Arrange
    mock_ticket_manager = MagicMock()
    mock_ticket_manager_class.return_value = mock_ticket_manager
    mock_ticket_manager.search_tickets_by_keyword.return_value = []
    
    # Act
    response = mcp_search_tickets({"keyword": "nonexistent"})
    
    # Assert
    assert "message" in response
    assert "No tickets found matching keyword: nonexistent" in response["message"]
    assert "tickets" in response
    assert response["tickets"] == []
    assert response["count"] == 0
```

```python
# In tests/test_fastmcp.py

@patch("jira_mcp.fastmcp_server.search_tickets")
def test_search_tickets_endpoint_success(self, mock_search, client):
    """Test the GET /tickets/search endpoint with successful search."""
    # Arrange
    mock_tickets = self._create_mock_tickets(2)
    mock_search.return_value = {
        "tickets": mock_tickets,
        "count": 2,
        "markdown": "Mock markdown table"
    }
    
    # Act
    response = client.get("/tickets/search?keyword=test&max_results=5")
    
    # Assert
    assert response.status_code == 200
    data = response.json()
    assert "tickets" in data
    assert data["count"] == 2
    assert "markdown" in data
    mock_search.assert_called_once_with({"keyword": "test", "max_results": 5})

@patch("jira_mcp.fastmcp_server.search_tickets")
def test_search_tickets_endpoint_error(self, mock_search, client):
    """Test the GET /tickets/search endpoint with an error."""
    # Arrange
    mock_search.side_effect = ValueError("No keyword provided")
    
    # Act
    response = client.get("/tickets/search")
    
    # Assert
    assert response.status_code == 400
    data = response.json()
    assert "detail" in data
    assert "No keyword provided" in data["detail"]
```

### 5. Update documentation

Update the API reference documentation to include the new tool and endpoint.

## Next Steps

After extending the server, make sure to:

1. Run the tests to ensure everything works as expected.
2. Update the documentation to reflect your changes.
3. Update the version number if appropriate.
4. Consider contributing your changes back to the main repository.