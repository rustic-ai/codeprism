"""Configuration management for the application."""

import os
import json
from typing import Any, Dict, Optional, Union
from pathlib import Path


class Config:
    """Configuration manager that loads from environment variables and files."""
    
    def __init__(self, config_file: Optional[str] = None):
        self.config_file = config_file
        self._config_data: Dict[str, Any] = {}
        self._load_config()
    
    def _load_config(self) -> None:
        """Load configuration from file and environment variables."""
        # Load from file if specified
        if self.config_file and os.path.exists(self.config_file):
            try:
                with open(self.config_file, 'r') as f:
                    self._config_data = json.load(f)
            except Exception as e:
                print(f"Warning: Failed to load config file {self.config_file}: {e}")
        
        # Environment variables override file config
        self._load_from_environment()
    
    def _load_from_environment(self) -> None:
        """Load configuration from environment variables."""
        # Common environment variables
        env_mappings = {
            "DEBUG": "debug",
            "DATABASE_URL": "database_url",
            "LOG_LEVEL": "log_level",
            "SECRET_KEY": "secret_key",
            "PORT": "port",
            "HOST": "host",
            "REDIS_URL": "redis_url",
            "CACHE_TTL": "cache_ttl",
            "MAX_CONNECTIONS": "max_connections",
            "TIMEOUT": "timeout"
        }
        
        for env_var, config_key in env_mappings.items():
            value = os.getenv(env_var)
            if value is not None:
                # Try to convert to appropriate type
                self._config_data[config_key] = self._convert_value(value)
    
    def _convert_value(self, value: str) -> Union[str, int, float, bool]:
        """Convert string value to appropriate type."""
        # Boolean conversion
        if value.lower() in ('true', 'false'):
            return value.lower() == 'true'
        
        # Integer conversion
        try:
            return int(value)
        except ValueError:
            pass
        
        # Float conversion
        try:
            return float(value)
        except ValueError:
            pass
        
        # Return as string
        return value
    
    def get(self, key: str, default: Any = None) -> Any:
        """Get a configuration value."""
        return self._config_data.get(key, default)
    
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
    
    def get_float(self, key: str, default: float = 0.0) -> float:
        """Get a float configuration value."""
        value = self.get(key, default)
        try:
            return float(value)
        except (ValueError, TypeError):
            return default
    
    def get_bool(self, key: str, default: bool = False) -> bool:
        """Get a boolean configuration value."""
        value = self.get(key, default)
        if isinstance(value, bool):
            return value
        if isinstance(value, str):
            return value.lower() in ('true', '1', 'yes', 'on')
        return bool(value) if value is not None else default
    
    def get_list(self, key: str, default: Optional[list] = None) -> list:
        """Get a list configuration value."""
        if default is None:
            default = []
        
        value = self.get(key, default)
        if isinstance(value, list):
            return value
        if isinstance(value, str):
            # Try to parse as JSON array
            try:
                parsed = json.loads(value)
                if isinstance(parsed, list):
                    return parsed
            except json.JSONDecodeError:
                pass
            # Split by comma as fallback
            return [item.strip() for item in value.split(',') if item.strip()]
        
        return default
    
    def set(self, key: str, value: Any) -> None:
        """Set a configuration value."""
        self._config_data[key] = value
    
    def update(self, config_dict: Dict[str, Any]) -> None:
        """Update configuration with a dictionary."""
        self._config_data.update(config_dict)
    
    def save_to_file(self, filename: str) -> bool:
        """Save current configuration to a file."""
        try:
            with open(filename, 'w') as f:
                json.dump(self._config_data, f, indent=2)
            return True
        except Exception as e:
            print(f"Failed to save config to {filename}: {e}")
            return False
    
    def reload(self) -> None:
        """Reload configuration from file and environment."""
        self._config_data.clear()
        self._load_config()
    
    def get_all(self) -> Dict[str, Any]:
        """Get all configuration values."""
        return self._config_data.copy()
    
    def has(self, key: str) -> bool:
        """Check if a configuration key exists."""
        return key in self._config_data
    
    def remove(self, key: str) -> bool:
        """Remove a configuration key."""
        if key in self._config_data:
            del self._config_data[key]
            return True
        return False
    
    def get_database_config(self) -> Dict[str, Any]:
        """Get database-specific configuration."""
        return {
            "url": self.get_string("database_url", "sqlite:///app.db"),
            "pool_size": self.get_int("db_pool_size", 5),
            "timeout": self.get_int("db_timeout", 30),
            "echo": self.get_bool("db_echo", False)
        }
    
    def get_logging_config(self) -> Dict[str, Any]:
        """Get logging-specific configuration."""
        return {
            "level": self.get_string("log_level", "INFO"),
            "file": self.get_string("log_file"),
            "format": self.get_string("log_format", "%(asctime)s - %(name)s - %(levelname)s - %(message)s"),
            "max_size": self.get_int("log_max_size", 10485760),  # 10MB
            "backup_count": self.get_int("log_backup_count", 5)
        }
    
    def get_security_config(self) -> Dict[str, Any]:
        """Get security-specific configuration."""
        return {
            "secret_key": self.get_string("secret_key", "dev-secret-key"),
            "session_timeout": self.get_int("session_timeout", 3600),
            "max_login_attempts": self.get_int("max_login_attempts", 5),
            "lockout_duration": self.get_int("lockout_duration", 300),
            "password_min_length": self.get_int("password_min_length", 8)
        }
    
    def validate_required_keys(self, required_keys: list) -> list:
        """Validate that required configuration keys are present."""
        missing_keys = []
        for key in required_keys:
            if not self.has(key):
                missing_keys.append(key)
        return missing_keys
    
    def __str__(self) -> str:
        """String representation of configuration (excluding sensitive data)."""
        safe_config = {}
        sensitive_keys = {'secret_key', 'password', 'token', 'key', 'api_key'}
        
        for key, value in self._config_data.items():
            if any(sensitive in key.lower() for sensitive in sensitive_keys):
                safe_config[key] = "***HIDDEN***"
            else:
                safe_config[key] = value
        
        return f"Config({safe_config})"


class EnvironmentConfig:
    """Environment-specific configuration loader."""
    
    def __init__(self, environment: str = "development"):
        self.environment = environment
        self.config = Config()
        self._load_environment_config()
    
    def _load_environment_config(self) -> None:
        """Load configuration specific to the environment."""
        config_files = [
            f"config/{self.environment}.json",
            f"config.{self.environment}.json",
            f"{self.environment}.config.json"
        ]
        
        for config_file in config_files:
            if os.path.exists(config_file):
                try:
                    with open(config_file, 'r') as f:
                        env_config = json.load(f)
                    self.config.update(env_config)
                    print(f"Loaded environment config from {config_file}")
                    break
                except Exception as e:
                    print(f"Failed to load {config_file}: {e}")
    
    def get_config(self) -> Config:
        """Get the loaded configuration."""
        return self.config


def load_config_from_file(filename: str) -> Config:
    """Load configuration from a specific file."""
    return Config(filename)


def create_default_config() -> Config:
    """Create a configuration with default values."""
    config = Config()
    config.update({
        "debug": False,
        "database_url": "sqlite:///app.db",
        "log_level": "INFO",
        "host": "localhost",
        "port": 8000,
        "secret_key": "dev-secret-key",
        "session_timeout": 3600,
        "max_login_attempts": 5,
        "lockout_duration": 300
    })
    return config


def get_environment() -> str:
    """Get the current environment from environment variables."""
    return os.getenv("ENVIRONMENT", os.getenv("ENV", "development")).lower()


def is_development() -> bool:
    """Check if running in development environment."""
    return get_environment() == "development"


def is_production() -> bool:
    """Check if running in production environment."""
    return get_environment() == "production"


def is_testing() -> bool:
    """Check if running in testing environment."""
    return get_environment() in ("test", "testing") 