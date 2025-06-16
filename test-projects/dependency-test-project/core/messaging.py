"""
Core messaging module for agent communication.
"""

from abc import ABC, abstractmethod
from dataclasses import dataclass
from enum import Enum
from typing import Any, Dict, Optional
from datetime import datetime


class Priority(Enum):
    """Message priority levels."""
    LOW = 1
    NORMAL = 2
    HIGH = 3
    URGENT = 4


@dataclass
class Message:
    """Core message class."""
    id: str
    type: str
    content: Any
    timestamp: str
    priority: Priority = Priority.NORMAL
    auth_token: Optional[str] = None
    requires_auth: bool = False
    updates_state: bool = False
    metadata: Dict[str, Any] = None
    
    def __post_init__(self):
        if self.metadata is None:
            self.metadata = {}


class MessageHandler(ABC):
    """Abstract message handler interface."""
    
    @abstractmethod
    async def handle(self, message: Message) -> Any:
        """Handle a message and return result."""
        pass
    
    @abstractmethod
    def can_handle(self, message_type: str) -> bool:
        """Check if this handler can process the message type."""
        pass


class DefaultMessageHandler(MessageHandler):
    """Default message handler implementation."""
    
    def __init__(self, supported_types: list[str]):
        self.supported_types = supported_types
    
    async def handle(self, message: Message) -> Any:
        """Handle a message."""
        return f"Processed message {message.id} of type {message.type}"
    
    def can_handle(self, message_type: str) -> bool:
        """Check if this handler can process the message type."""
        return message_type in self.supported_types 