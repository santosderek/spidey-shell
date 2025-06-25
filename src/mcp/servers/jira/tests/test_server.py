"""
Tests for the MCP server module.
"""

from unittest.mock import MagicMock, patch

from jira_mcp.server import process_message


class TestJiraMCPServer:
    """Test case for the MCP server functionality."""

    @patch("jira_mcp.server.MCP_TOOLS")
    def test_process_message_known_command(self, mock_tools):
        """Test processing a known command."""
        # Arrange
        mock_function = MagicMock()
        mock_function.return_value = {"result": "success"}
        mock_tools.__getitem__.return_value = {
            "function": mock_function,
            "description": "Test command",
        }
        mock_tools.__contains__.return_value = True
        message = {"command": "jira.test-command"}

        # Act
        result = process_message(message)

        # Assert
        assert result == {"result": "success"}
        mock_function.assert_called_once_with(message)

    @patch("jira_mcp.server.MCP_TOOLS")
    def test_process_message_unknown_command(self, mock_tools):
        """Test processing an unknown command."""
        # Arrange
        mock_tools.__contains__.return_value = False
        mock_tools.keys.return_value = ["jira.command1", "jira.command2"]
        message = {"command": "unknown-command"}

        # Act
        result = process_message(message)

        # Assert
        assert "error" in result
        assert "Unknown command" in result["error"]
        assert "available_commands" in result

    @patch("jira_mcp.server.MCP_TOOLS")
    def test_process_message_list_commands(self, mock_tools):
        """Test the list-commands command."""
        # Arrange
        mock_tools.items.return_value = [
            (
                "jira.command1",
                {"description": "Command 1", "args": {"arg1": {"description": "Arg 1"}}},
            ),
            (
                "jira.command2",
                {"description": "Command 2", "args": {}},
            ),
        ]
        message = {"command": "list-commands"}

        # Act
        result = process_message(message)

        # Assert
        assert "commands" in result
        assert len(result["commands"]) == 2
        assert result["commands"][0]["name"] == "jira.command1"
        assert result["commands"][1]["name"] == "jira.command2"

    @patch("jira_mcp.server.MCP_TOOLS")
    def test_process_message_error(self, mock_tools):
        """Test error handling in process_message."""
        # Arrange
        mock_function = MagicMock()
        mock_function.side_effect = Exception("Test error")
        mock_tools.__getitem__.return_value = {
            "function": mock_function,
            "description": "Test command",
        }
        mock_tools.__contains__.return_value = True
        message = {"command": "jira.error-command"}

        # Act
        result = process_message(message)

        # Assert
        assert "error" in result
        assert "Internal server error" in result["error"]
        assert "Test error" in result["error"]