"""
Configuration management utilities.
"""

import os
from typing import Any, Dict, Optional


class ConfigManager:
    """Manages application configuration."""
    
    def __init__(self, config_file: Optional[str] = None):
        self.config_file = config_file
        self._config: Dict[str, Any] = self._load_config()
    
    def get(self, key: str, default: Any = None) -> Any:
        """Get a configuration value."""
        return self._config.get(key, default)
    
    def set(self, key: str, value: Any) -> None:
        """Set a configuration value."""
        self._config[key] = value
    
    def get_string(self, key: str, default: str = "") -> str:
        """Get a string configuration value."""
        value = self.get(key, default)
        return str(value) if value is not None else default
    
    def get_int(self, key: str, default: int = 0) -> int:
        """Get an integer configuration value."""
        value = self.get(key, default)
        try:
            return int(value)
        except (ValueError, TypeError):
            return default
    
    def get_bool(self, key: str, default: bool = False) -> bool:
        """Get a boolean configuration value."""
        value = self.get(key, default)
        if isinstance(value, bool):
            return value
        if isinstance(value, str):
            return value.lower() in ('true', '1', 'yes', 'on')
        return default
    
    def _load_config(self) -> Dict[str, Any]:
        """Load configuration from environment and file."""
        config = {}
        
        # Load from environment variables
        for key, value in os.environ.items():
            if key.startswith('AGENT_'):
                config_key = key[6:].lower()  # Remove 'AGENT_' prefix
                config[config_key] = value
        
        # Add some defaults
        config.setdefault('debug', 'false')
        config.setdefault('log_level', 'INFO')
        config.setdefault('database_url', 'sqlite://memory')
        
        return config 