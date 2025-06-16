"""
State management module for agents.
"""

from dataclasses import dataclass
from typing import Any, Dict, Optional
import asyncio


@dataclass
class StateUpdate:
    """Represents a state update."""
    agent_id: str
    message_id: str
    result: Any
    timestamp: str
    metadata: Dict[str, Any] = None
    
    def __post_init__(self):
        if self.metadata is None:
            self.metadata = {}


class StateManager:
    """Manages agent state."""
    
    def __init__(self):
        self._state: Dict[str, Any] = {}
        self._history: list[StateUpdate] = []
        self._initialized = False
    
    async def initialize(self) -> None:
        """Initialize the state manager."""
        self._initialized = True
    
    async def update_state(self, update: StateUpdate) -> None:
        """Update agent state."""
        if not self._initialized:
            raise RuntimeError("StateManager not initialized")
        
        self._state[update.agent_id] = update.result
        self._history.append(update)
    
    async def get_state(self, agent_id: str) -> Optional[Any]:
        """Get current state for an agent."""
        return self._state.get(agent_id)
    
    async def get_history(self, agent_id: str) -> list[StateUpdate]:
        """Get state history for an agent."""
        return [update for update in self._history if update.agent_id == agent_id]
    
    async def health_check(self) -> Dict[str, Any]:
        """Health check for state manager."""
        return {
            "status": "healthy" if self._initialized else "not_initialized",
            "agents_tracked": len(self._state),
            "history_size": len(self._history)
        }
    
    async def shutdown(self) -> None:
        """Shutdown the state manager."""
        self._initialized = False
        self._state.clear()
        self._history.clear() 