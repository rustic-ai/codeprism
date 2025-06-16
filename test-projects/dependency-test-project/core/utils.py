"""
Core utilities for the agent system.
"""

import asyncio
import logging
import time
import uuid
from typing import Callable, Optional


class IDGenerator:
    """Generates unique IDs for agents and messages."""
    
    @staticmethod
    def generate_id() -> str:
        """Generate a unique ID."""
        return str(uuid.uuid4())
    
    @staticmethod
    def generate_short_id() -> str:
        """Generate a shorter unique ID."""
        return str(uuid.uuid4())[:8]


class Timer:
    """Simple timer for periodic tasks."""
    
    def __init__(self, interval: float, callback: Callable):
        self.interval = interval
        self.callback = callback
        self._task: Optional[asyncio.Task] = None
        self._running = False
    
    def start(self) -> None:
        """Start the timer."""
        if not self._running:
            self._running = True
            self._task = asyncio.create_task(self._run())
    
    def stop(self) -> None:
        """Stop the timer."""
        if self._running and self._task:
            self._running = False
            self._task.cancel()
    
    async def _run(self) -> None:
        """Internal timer loop."""
        while self._running:
            try:
                await asyncio.sleep(self.interval)
                if self._running:
                    if asyncio.iscoroutinefunction(self.callback):
                        await self.callback()
                    else:
                        self.callback()
            except asyncio.CancelledError:
                break
            except Exception as e:
                print(f"Timer callback error: {e}")


class LoggerMixin:
    """Mixin to add logging capabilities to classes."""
    
    def __init__(self):
        self.logger = logging.getLogger(self.__class__.__name__)
    
    def log_info(self, message: str) -> None:
        """Log an info message."""
        self.logger.info(message)
    
    def log_debug(self, message: str) -> None:
        """Log a debug message."""
        self.logger.debug(message)
    
    def log_error(self, message: str) -> None:
        """Log an error message."""
        self.logger.error(message)
    
    def log_warning(self, message: str) -> None:
        """Log a warning message."""
        self.logger.warning(message) 