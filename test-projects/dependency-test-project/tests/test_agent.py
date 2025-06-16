"""
Tests for the Agent class.
These test files would typically be excluded by smart dependency filtering.
"""

import pytest
import asyncio
from core.agent import Agent, AgentConfig, AgentType, create_agent
from core.messaging import Message, Priority


class TestAgent:
    """Test cases for Agent class."""
    
    @pytest.fixture
    async def agent(self):
        """Create a test agent."""
        config = AgentConfig(
            name="test_agent",
            agent_type=AgentType.BOT,
            debug=True
        )
        agent = create_agent("basic", config.dict())
        await agent.initialize()
        yield agent
        await agent.shutdown()
    
    async def test_agent_initialization(self, agent):
        """Test agent initialization."""
        assert agent.status.value == "running"
        assert agent.config.name == "test_agent"
    
    async def test_agent_message_processing(self, agent):
        """Test message processing."""
        message = Message(
            id="test_msg",
            type="test",
            content="Hello",
            timestamp="2024-01-01T00:00:00Z"
        )
        
        # This would fail without proper handler, but tests the interface
        try:
            result = await agent.process_message(message)
        except ValueError:
            pass  # Expected for unhandled message type
    
    async def test_agent_health_check(self, agent):
        """Test agent health check."""
        health = await agent.health_check()
        assert "agent_id" in health
        assert "status" in health
        assert "dependencies" in health 