"""
Jira Mock Server Implementation.

This module provides a FastAPI server implementation that mocks a Jira instance 
for testing and development purposes.
"""
import logging
from typing import Any, Dict, Optional

from fastapi import FastAPI, Request, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse

from jira_mcp.settings import settings
from tests.mock_server.api import api_router

logger = logging.getLogger(__name__)

# Create FastAPI app
app = FastAPI(
    title="Jira Mock Server",
    description="A mock server that emulates the Jira REST API",
    version="1.0.0",
)

# Add CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Authentication middleware
@app.middleware("http")
async def auth_middleware(request: Request, call_next: Any):
    """Basic auth middleware for the mock server."""
    # Skip auth for serverInfo and other public endpoints
    if request.url.path == "/rest/api/2/serverInfo":
        return await call_next(request)
        
    # Check for Authorization header
    auth_header = request.headers.get("Authorization")
    if not auth_header or not auth_header.startswith("Basic "):
        return JSONResponse(
            status_code=401,
            content={"message": "Unauthorized: No valid authentication credentials"},
            headers={"WWW-Authenticate": "Basic"},
        )
        
    # In a real implementation, you'd verify the credentials
    # For this mock, we'll accept any credentials
    
    return await call_next(request)

# Include API routers
app.include_router(api_router)

# Root endpoint
@app.get("/")
async def root():
    """Root endpoint for the Jira mock server."""
    return {
        "name": "Jira Mock Server", 
        "version": "1.0.0",
        "status": "running"
    }

# Health check endpoint
@app.get("/health")
async def health_check():
    """Health check endpoint for the Jira mock server."""
    return {"status": "healthy"}

# Error handler
@app.exception_handler(HTTPException)
async def http_exception_handler(request: Request, exc: HTTPException):
    """Custom HTTP exception handler."""
    return JSONResponse(
        status_code=exc.status_code,
        content={"message": exc.detail, "error": True},
    )

# General exception handler
@app.exception_handler(Exception)
async def general_exception_handler(request: Request, exc: Exception):
    """General exception handler for unexpected errors."""
    logger.exception("Unhandled exception")
    return JSONResponse(
        status_code=500,
        content={"message": "Internal server error", "error": True},
    )


def run_server(host: str = "0.0.0.0", port: int = 8000, debug: bool = False):
    """
    Run the Jira mock server.
    
    Args:
        host: Host to listen on
        port: Port to listen on
        debug: Whether to run in debug mode
    """
    import uvicorn
    
    logger.info(f"Starting Jira mock server on {host}:{port}")
    uvicorn.run(
        "tests.mock_server.jira_mock:app",  # Updated import path
        host=host,
        port=port,
        reload=debug,
        log_level="debug" if debug else "info",
    )


if __name__ == "__main__":
    run_server(debug=settings.debug)