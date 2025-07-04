"""
Tests for WebSocket communication.

These tests verify that the WebSocket functionality
using mocks instead of actual connections.
"""

import pytest
from unittest.mock import MagicMock, AsyncMock

from tests.mock_server.server import ConnectionManager, WebSocket, WebSocketDisconnect


class TestWebSockets:
    """WebSocket tests using mocks."""
    
    @pytest.mark.asyncio
    async def test_connection_manager_connect(self):
        """Test connection manager's connect method."""
        # Given a connection manager and a mock WebSocket
        manager = ConnectionManager()
        websocket = AsyncMock(spec=WebSocket)
        
        # When we connect a WebSocket
        await manager.connect(websocket)
        
        # Then the WebSocket should be accepted and added to active connections
        websocket.accept.assert_called_once()
        assert websocket in manager.active_connections
    
    @pytest.mark.asyncio
    async def test_connection_manager_disconnect(self):
        """Test connection manager's disconnect method."""
        # Given a connection manager with a connected WebSocket
        manager = ConnectionManager()
        websocket = AsyncMock(spec=WebSocket)
        await manager.connect(websocket)
        
        # When we disconnect the WebSocket
        manager.disconnect(websocket)
        
        # Then the WebSocket should be removed from active connections
        assert websocket not in manager.active_connections
    
    @pytest.mark.asyncio
    async def test_connection_manager_broadcast(self):
        """Test connection manager's broadcast method."""
        # Given a connection manager with multiple connected WebSockets
        manager = ConnectionManager()
        websockets = [AsyncMock(spec=WebSocket) for _ in range(3)]
        
        for ws in websockets:
            await manager.connect(ws)
        
        # When we broadcast a message
        test_data = {"message": "Test broadcast"}
        await manager.broadcast(test_data)
        
        # Then all WebSockets should receive the message
        for ws in websockets:
            ws.send_json.assert_called_once_with(test_data)
    
    @pytest.mark.asyncio
    async def test_connection_manager_broadcast_with_dead_connection(self):
        """Test connection manager's broadcast with a dead connection."""
        # Given a connection manager with multiple WebSockets, one of which is dead
        manager = ConnectionManager()
        websockets = [AsyncMock(spec=WebSocket) for _ in range(3)]
        
        for ws in websockets:
            await manager.connect(ws)
        
        # Make one connection fail when sending
        websockets[1].send_json.side_effect = Exception("Connection lost")
        
        # When we broadcast a message
        test_data = {"message": "Test broadcast"}
        await manager.broadcast(test_data)
        
        # Then the dead connection should be handled gracefully
        assert len(manager.active_connections) == 2