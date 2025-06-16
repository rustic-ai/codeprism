#!/usr/bin/env python3
"""
Core Agent class that demonstrates realistic dependency usage patterns.
This file mimics the structure from your rustic-ai project.
"""

from __future__ import annotations

import asyncio
import inspect
import logging
from abc import ABC, abstractmethod
from copy import deepcopy
from enum import Enum
from typing import Any, Callable, Dict, Generic, List, Optional, Type, TypeVar

# External dependencies (would be in venv/.tox in real projects)
from pydantic import BaseModel, Field
from fastapi import FastAPI
from requests import Session
from sqlalchemy import Column, Integer, String
from dataclasses import dataclass

# Internal project imports
from core.messaging import Message, MessageHandler, Priority
from core.state import StateManager, StateUpdate
from core.utils import IDGenerator, Timer
from services.auth import AuthProvider
from services.database import DatabaseAdapter
from utils.config import ConfigManager
from utils.logger import LoggerMixin


class AgentType(Enum):
    """Agent type enumeration."""
    HUMAN = "human"
    BOT = "bot"
    SYSTEM = "system"


class AgentMode(Enum):
    """Agent execution mode."""
    LOCAL = "local"
    REMOTE = "remote"
    DISTRIBUTED = "distributed"


class AgentStatus(Enum):
    """Agent status enumeration."""
    IDLE = "idle"
    RUNNING = "running"
    PAUSED = "paused"
    STOPPED = "stopped"
    ERROR = "error"


@dataclass
class AgentConfig:
    """Agent configuration dataclass."""
    name: str
    agent_type: AgentType = AgentType.BOT
    mode: AgentMode = AgentMode.LOCAL
    max_retries: int = 3
    timeout: float = 30.0
    debug: bool = False


class AgentMetrics(BaseModel):
    """Agent metrics using Pydantic for validation."""
    messages_processed: int = Field(default=0, ge=0)
    errors_count: int = Field(default=0, ge=0)
    uptime_seconds: float = Field(default=0.0, ge=0.0)
    last_activity: Optional[str] = None
    
    class Config:
        """Pydantic configuration."""
        validate_assignment = True


class AgentInterface(ABC):
    """Abstract base class for all agents."""
    
    @abstractmethod
    async def initialize(self) -> bool:
        """Initialize the agent."""
        pass
    
    @abstractmethod
    async def process_message(self, message: Message) -> Any:
        """Process an incoming message."""
        pass
    
    @abstractmethod
    async def shutdown(self) -> None:
        """Shutdown the agent gracefully."""
        pass


class Agent(AgentInterface, LoggerMixin, Generic[TypeVar('T')]):
    """
    Core Agent class demonstrating complex dependency patterns.
    
    This class shows realistic usage of:
    - External dependencies (pydantic, fastapi, requests, sqlalchemy)
    - Internal project modules
    - Complex inheritance and composition
    - Generic types and type hints
    - Async/await patterns
    """
    
    def __init__(
        self,
        config: AgentConfig,
        auth_provider: Optional[AuthProvider] = None,
        db_adapter: Optional[DatabaseAdapter] = None,
        state_manager: Optional[StateManager] = None,
    ):
        """Initialize the agent with dependency injection."""
        super().__init__()
        
        self.config = config
        self.id = IDGenerator.generate_id()
        self.status = AgentStatus.IDLE
        self.metrics = AgentMetrics()
        
        # Dependency injection
        self.auth_provider = auth_provider or AuthProvider()
        self.db_adapter = db_adapter or DatabaseAdapter()
        self.state_manager = state_manager or StateManager()
        
        # Internal state
        self._message_handlers: Dict[str, MessageHandler] = {}
        self._timers: List[Timer] = []
        self._session = Session()  # HTTP session for external calls
        self._app = FastAPI(title=f"Agent-{self.config.name}")  # FastAPI instance
        
        # Initialize logger
        self.logger = logging.getLogger(f"agent.{self.config.name}")
        
    async def initialize(self) -> bool:
        """Initialize the agent and its dependencies."""
        try:
            self.log_info("Initializing agent...")
            
            # Initialize dependencies
            await self.auth_provider.initialize()
            await self.db_adapter.connect()
            await self.state_manager.initialize()
            
            # Set up message handlers
            self._setup_message_handlers()
            
            # Start internal timers
            self._start_timers()
            
            self.status = AgentStatus.RUNNING
            self.log_info("Agent initialized successfully")
            return True
            
        except Exception as e:
            self.log_error(f"Failed to initialize agent: {e}")
            self.status = AgentStatus.ERROR
            return False
    
    async def process_message(self, message: Message) -> Any:
        """Process an incoming message."""
        if self.status != AgentStatus.RUNNING:
            raise ValueError(f"Agent not running (status: {self.status})")
        
        try:
            # Update metrics
            self.metrics.messages_processed += 1
            self.metrics.last_activity = message.timestamp
            
            # Authenticate message if needed
            if message.requires_auth and not await self._authenticate_message(message):
                raise PermissionError("Message authentication failed")
            
            # Find appropriate handler
            handler = self._get_message_handler(message.type)
            if not handler:
                raise ValueError(f"No handler for message type: {message.type}")
            
            # Process the message
            result = await handler.handle(message)
            
            # Update state if needed
            if message.updates_state:
                await self._update_state(message, result)
            
            # Log successful processing
            self.log_debug(f"Processed message {message.id} successfully")
            
            return result
            
        except Exception as e:
            self.metrics.errors_count += 1
            self.log_error(f"Error processing message {message.id}: {e}")
            raise
    
    async def shutdown(self) -> None:
        """Shutdown the agent gracefully."""
        self.log_info("Shutting down agent...")
        
        self.status = AgentStatus.STOPPED
        
        # Stop timers
        for timer in self._timers:
            timer.stop()
        
        # Close HTTP session
        self._session.close()
        
        # Shutdown dependencies
        await self.db_adapter.disconnect()
        await self.state_manager.shutdown()
        
        self.log_info("Agent shutdown complete")
    
    def register_message_handler(self, message_type: str, handler: MessageHandler) -> None:
        """Register a message handler for a specific message type."""
        self._message_handlers[message_type] = handler
        self.log_debug(f"Registered handler for message type: {message_type}")
    
    def get_metrics(self) -> AgentMetrics:
        """Get current agent metrics."""
        return deepcopy(self.metrics)
    
    def get_status(self) -> AgentStatus:
        """Get current agent status."""
        return self.status
    
    async def health_check(self) -> Dict[str, Any]:
        """Perform a health check of the agent and its dependencies."""
        health = {
            "agent_id": self.id,
            "status": self.status.value,
            "metrics": self.metrics.dict(),
            "dependencies": {}
        }
        
        # Check auth provider
        try:
            health["dependencies"]["auth"] = await self.auth_provider.health_check()
        except Exception as e:
            health["dependencies"]["auth"] = {"status": "error", "error": str(e)}
        
        # Check database
        try:
            health["dependencies"]["database"] = await self.db_adapter.health_check()
        except Exception as e:
            health["dependencies"]["database"] = {"status": "error", "error": str(e)}
        
        # Check state manager
        try:
            health["dependencies"]["state"] = await self.state_manager.health_check()
        except Exception as e:
            health["dependencies"]["state"] = {"status": "error", "error": str(e)}
        
        return health
    
    async def _authenticate_message(self, message: Message) -> bool:
        """Authenticate a message using the auth provider."""
        if not message.auth_token:
            return False
        
        return await self.auth_provider.validate_token(message.auth_token)
    
    def _get_message_handler(self, message_type: str) -> Optional[MessageHandler]:
        """Get the appropriate message handler for a message type."""
        return self._message_handlers.get(message_type)
    
    async def _update_state(self, message: Message, result: Any) -> None:
        """Update agent state based on message processing result."""
        state_update = StateUpdate(
            agent_id=self.id,
            message_id=message.id,
            result=result,
            timestamp=message.timestamp
        )
        
        await self.state_manager.update_state(state_update)
    
    def _setup_message_handlers(self) -> None:
        """Set up default message handlers."""
        # This would typically be implemented by subclasses
        self.log_debug("Setting up default message handlers")
    
    def _start_timers(self) -> None:
        """Start internal timers for periodic tasks."""
        # Metrics update timer
        metrics_timer = Timer(
            interval=60.0,  # Update metrics every minute
            callback=self._update_metrics
        )
        self._timers.append(metrics_timer)
        metrics_timer.start()
    
    async def _update_metrics(self) -> None:
        """Update agent metrics periodically."""
        # This would update uptime, performance metrics, etc.
        self.metrics.uptime_seconds += 60.0


class SpecializedAgent(Agent[str]):
    """
    A specialized agent demonstrating inheritance and specific typing.
    """
    
    def __init__(self, config: AgentConfig, **kwargs):
        super().__init__(config, **kwargs)
        self.specialized_feature = "custom_processing"
    
    async def specialized_method(self, data: str) -> str:
        """A method specific to this agent type."""
        self.log_info(f"Processing specialized data: {data}")
        return f"processed_{data}"


# Factory function demonstrating complex instantiation
def create_agent(
    agent_type: str,
    config: Dict[str, Any],
    dependencies: Optional[Dict[str, Any]] = None
) -> Agent:
    """Factory function to create different types of agents."""
    
    agent_config = AgentConfig(**config)
    deps = dependencies or {}
    
    if agent_type == "specialized":
        return SpecializedAgent(
            config=agent_config,
            auth_provider=deps.get("auth_provider"),
            db_adapter=deps.get("db_adapter"),
            state_manager=deps.get("state_manager")
        )
    else:
        return Agent(
            config=agent_config,
            auth_provider=deps.get("auth_provider"),
            db_adapter=deps.get("db_adapter"),
            state_manager=deps.get("state_manager")
        )


# Example usage and integration patterns
async def example_usage():
    """Example of how the Agent class would be used."""
    
    # Create configuration
    config = AgentConfig(
        name="example_agent",
        agent_type=AgentType.BOT,
        mode=AgentMode.LOCAL,
        debug=True
    )
    
    # Create agent
    agent = create_agent("specialized", config.dict())
    
    # Initialize and use
    if await agent.initialize():
        # Create and process a message
        message = Message(
            id="msg_001",
            type="test",
            content="Hello, Agent!",
            timestamp="2024-01-01T00:00:00Z"
        )
        
        result = await agent.process_message(message)
        print(f"Message processing result: {result}")
        
        # Check health
        health = await agent.health_check()
        print(f"Agent health: {health}")
        
        # Shutdown
        await agent.shutdown()


if __name__ == "__main__":
    asyncio.run(example_usage()) 