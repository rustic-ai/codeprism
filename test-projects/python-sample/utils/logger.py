"""Logging utility for the application."""

import logging
import sys
from datetime import datetime
from typing import Optional, Dict, Any
from enum import Enum


class LogLevel(Enum):
    """Log level enumeration."""
    DEBUG = "DEBUG"
    INFO = "INFO"
    WARNING = "WARNING"
    ERROR = "ERROR"
    CRITICAL = "CRITICAL"


class Logger:
    """Application logger with various output formats."""
    
    def __init__(self, level: str = "INFO", log_file: Optional[str] = None):
        self.level = level.upper()
        self.log_file = log_file
        self._setup_logger()
        
    def _setup_logger(self) -> None:
        """Set up the logger configuration."""
        self.logger = logging.getLogger("app_logger")
        self.logger.setLevel(getattr(logging, self.level))
        
        # Clear existing handlers
        self.logger.handlers.clear()
        
        # Create formatter
        formatter = logging.Formatter(
            '%(asctime)s - %(name)s - %(levelname)s - %(message)s'
        )
        
        # Console handler
        console_handler = logging.StreamHandler(sys.stdout)
        console_handler.setFormatter(formatter)
        self.logger.addHandler(console_handler)
        
        # File handler if specified
        if self.log_file:
            file_handler = logging.FileHandler(self.log_file)
            file_handler.setFormatter(formatter)
            self.logger.addHandler(file_handler)
    
    def debug(self, message: str, **kwargs) -> None:
        """Log a debug message."""
        self._log(LogLevel.DEBUG, message, **kwargs)
    
    def info(self, message: str, **kwargs) -> None:
        """Log an info message."""
        self._log(LogLevel.INFO, message, **kwargs)
    
    def warning(self, message: str, **kwargs) -> None:
        """Log a warning message."""
        self._log(LogLevel.WARNING, message, **kwargs)
    
    def error(self, message: str, **kwargs) -> None:
        """Log an error message."""
        self._log(LogLevel.ERROR, message, **kwargs)
    
    def critical(self, message: str, **kwargs) -> None:
        """Log a critical message."""
        self._log(LogLevel.CRITICAL, message, **kwargs)
    
    def _log(self, level: LogLevel, message: str, **kwargs) -> None:
        """Internal logging method."""
        # Add extra context if provided
        extra_info = ""
        if kwargs:
            extra_info = f" | Extra: {kwargs}"
        
        full_message = f"{message}{extra_info}"
        
        # Log using the appropriate level
        if level == LogLevel.DEBUG:
            self.logger.debug(full_message)
        elif level == LogLevel.INFO:
            self.logger.info(full_message)
        elif level == LogLevel.WARNING:
            self.logger.warning(full_message)
        elif level == LogLevel.ERROR:
            self.logger.error(full_message)
        elif level == LogLevel.CRITICAL:
            self.logger.critical(full_message)
    
    def log_exception(self, message: str, exception: Exception) -> None:
        """Log an exception with traceback."""
        self.logger.error(f"{message}: {str(exception)}", exc_info=True)
    
    def log_user_action(self, username: str, action: str, details: Optional[Dict[str, Any]] = None) -> None:
        """Log a user action."""
        log_message = f"User '{username}' performed action: {action}"
        if details:
            log_message += f" | Details: {details}"
        self.info(log_message)
    
    def log_performance(self, operation: str, duration_ms: float, **metrics) -> None:
        """Log performance metrics."""
        message = f"Performance: {operation} took {duration_ms:.2f}ms"
        if metrics:
            message += f" | Metrics: {metrics}"
        self.info(message)
    
    def log_security_event(self, event_type: str, details: Dict[str, Any]) -> None:
        """Log a security-related event."""
        message = f"Security Event: {event_type} | Details: {details}"
        self.warning(message)
    
    def set_level(self, level: str) -> None:
        """Change the logging level."""
        self.level = level.upper()
        self.logger.setLevel(getattr(logging, self.level))
    
    def add_file_handler(self, filename: str) -> None:
        """Add a file handler to the logger."""
        formatter = logging.Formatter(
            '%(asctime)s - %(name)s - %(levelname)s - %(message)s'
        )
        file_handler = logging.FileHandler(filename)
        file_handler.setFormatter(formatter)
        self.logger.addHandler(file_handler)
    
    def get_log_stats(self) -> Dict[str, Any]:
        """Get logging statistics."""
        return {
            "current_level": self.level,
            "handlers_count": len(self.logger.handlers),
            "log_file": self.log_file,
            "logger_name": self.logger.name
        }


class StructuredLogger:
    """Structured logger that outputs JSON format."""
    
    def __init__(self, service_name: str):
        self.service_name = service_name
        
    def log(self, level: str, message: str, **context) -> None:
        """Log a structured message."""
        log_entry = {
            "timestamp": datetime.now().isoformat(),
            "service": self.service_name,
            "level": level.upper(),
            "message": message,
            "context": context
        }
        
        # In a real application, this would be sent to a log aggregation service
        print(f"STRUCTURED_LOG: {log_entry}")
    
    def info(self, message: str, **context) -> None:
        """Log structured info message."""
        self.log("INFO", message, **context)
    
    def error(self, message: str, **context) -> None:
        """Log structured error message."""
        self.log("ERROR", message, **context)
    
    def debug(self, message: str, **context) -> None:
        """Log structured debug message."""
        self.log("DEBUG", message, **context)


def create_logger(name: str, level: str = "INFO", log_file: Optional[str] = None) -> Logger:
    """Factory function to create a logger."""
    return Logger(level, log_file)


def log_function_call(func):
    """Decorator to log function calls."""
    def wrapper(*args, **kwargs):
        logger = Logger()
        logger.debug(f"Calling function: {func.__name__} with args: {args}, kwargs: {kwargs}")
        
        try:
            result = func(*args, **kwargs)
            logger.debug(f"Function {func.__name__} completed successfully")
            return result
        except Exception as e:
            logger.error(f"Function {func.__name__} failed with error: {e}")
            raise
    
    return wrapper


def measure_performance(func):
    """Decorator to measure and log function performance."""
    def wrapper(*args, **kwargs):
        logger = Logger()
        start_time = datetime.now()
        
        try:
            result = func(*args, **kwargs)
            end_time = datetime.now()
            duration = (end_time - start_time).total_seconds() * 1000
            logger.log_performance(func.__name__, duration)
            return result
        except Exception as e:
            end_time = datetime.now()
            duration = (end_time - start_time).total_seconds() * 1000
            logger.log_performance(func.__name__, duration, error=str(e))
            raise
    
    return wrapper 