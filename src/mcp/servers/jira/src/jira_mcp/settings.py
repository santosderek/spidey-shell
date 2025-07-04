"""
Settings management for Jira MCP using pydantic-settings.

This module provides a centralized location for all configuration settings
used by the Jira MCP server, with sensible defaults and environment variable support.
"""

from pydantic_settings import BaseSettings, SettingsConfigDict
from typing import Optional


class JiraSettings(BaseSettings):
    """Jira API connection settings."""
    
    # Connection settings
    jira_url: str = "https://cisco-jira.atlassian.net"
    jira_username: Optional[str] = None
    jira_api_token: Optional[str] = None
    
    # Search settings
    default_days_lookback: int = 30
    max_results_per_query: int = 50
    
    # MCP Server settings
    mcp_name: str = "JiraMCP"
    debug: bool = False
    
    # Mock Server settings
    mock_server_host: str = "0.0.0.0"
    mock_server_port: int = 8000
    mock_version: str = "9.0.0"
    
    # Configure env file and variable names
    model_config = SettingsConfigDict(
        env_file=".env",
        env_file_encoding="utf-8",
        env_prefix="JIRA_MCP_",
        extra="ignore",
    )
    
    @property
    def has_credentials(self) -> bool:
        """Check if credentials are configured."""
        return bool(self.jira_username and self.jira_api_token)
    
    @property
    def credentials_tuple(self) -> Optional[tuple[str, str]]:
        """Return credentials as a tuple for Jira client."""
        if self.has_credentials:
            return (self.jira_username, self.jira_api_token)
        return None


# Create a global settings instance
settings = JiraSettings()