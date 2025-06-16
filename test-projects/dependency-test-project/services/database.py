"""
Database service for agent system.
"""

import asyncio
from typing import Any, Dict, List, Optional


class DatabaseAdapter:
    """Database adapter for agent persistence."""
    
    def __init__(self, connection_string: str = "sqlite://memory"):
        self.connection_string = connection_string
        self._connected = False
        self._data: Dict[str, Any] = {}
    
    async def connect(self) -> None:
        """Connect to the database."""
        # Simulate connection delay
        await asyncio.sleep(0.1)
        self._connected = True
    
    async def disconnect(self) -> None:
        """Disconnect from the database."""
        self._connected = False
        self._data.clear()
    
    async def save(self, table: str, key: str, data: Any) -> bool:
        """Save data to the database."""
        if not self._connected:
            raise RuntimeError("Database not connected")
        
        if table not in self._data:
            self._data[table] = {}
        
        self._data[table][key] = data
        return True
    
    async def load(self, table: str, key: str) -> Optional[Any]:
        """Load data from the database."""
        if not self._connected:
            raise RuntimeError("Database not connected")
        
        return self._data.get(table, {}).get(key)
    
    async def query(self, table: str, filters: Dict[str, Any] = None) -> List[Any]:
        """Query data from the database."""
        if not self._connected:
            raise RuntimeError("Database not connected")
        
        table_data = self._data.get(table, {})
        results = list(table_data.values())
        
        # Simple filtering
        if filters:
            filtered_results = []
            for item in results:
                match = True
                for key, value in filters.items():
                    if isinstance(item, dict) and item.get(key) != value:
                        match = False
                        break
                if match:
                    filtered_results.append(item)
            return filtered_results
        
        return results
    
    async def health_check(self) -> Dict[str, str]:
        """Health check for database."""
        return {
            "status": "connected" if self._connected else "disconnected",
            "tables": str(len(self._data))
        } 