import logging

from mcp.server.fastmcp import FastMCP


# Configure logging
logger = logging.getLogger(__name__)


# Create FastMCP server
mcp = FastMCP("JiraMCP")


# Import submodules to register tools
from . import tools
from . import manager
