#!/usr/bin/env python3
"""
Main application entry point for the sample Python project.
This demonstrates various code patterns for testing the MCP server.
"""

import os
import sys
from typing import List, Dict, Optional
from dataclasses import dataclass

from models.user import User, UserManager
from services.auth import AuthService
from services.database import DatabaseService
from utils.logger import Logger
from utils.config import Config


@dataclass
class AppConfig:
    """Application configuration."""
    debug: bool = False
    database_url: str = "sqlite:///app.db"
    log_level: str = "INFO"


class Application:
    """Main application class."""
    
    def __init__(self, config: AppConfig):
        self.config = config
        self.logger = Logger(config.log_level)
        self.db_service = DatabaseService(config.database_url)
        self.auth_service = AuthService(self.db_service)
        self.user_manager = UserManager(self.db_service)
        
    def initialize(self) -> bool:
        """Initialize the application."""
        try:
            self.logger.info("Initializing application...")
            self.db_service.connect()
            self.db_service.create_tables()
            self.logger.info("Application initialized successfully")
            return True
        except Exception as e:
            self.logger.error(f"Failed to initialize application: {e}")
            return False
    
    def run(self) -> None:
        """Run the main application loop."""
        if not self.initialize():
            sys.exit(1)
            
        self.logger.info("Starting application...")
        
        # Create some sample users
        users = self.create_sample_users()
        
        # Demonstrate authentication
        self.demonstrate_auth(users)
        
        # Show user management
        self.demonstrate_user_management()
        
        self.logger.info("Application finished")
    
    def create_sample_users(self) -> List[User]:
        """Create sample users for demonstration."""
        users = [
            User(username="alice", email="alice@example.com", age=30),
            User(username="bob", email="bob@example.com", age=25),
            User(username="charlie", email="charlie@example.com", age=35),
        ]
        
        for user in users:
            self.user_manager.create_user(user)
            self.logger.info(f"Created user: {user.username}")
            
        return users
    
    def demonstrate_auth(self, users: List[User]) -> None:
        """Demonstrate authentication functionality."""
        for user in users:
            # Set password
            self.auth_service.set_password(user.username, "password123")
            
            # Authenticate
            if self.auth_service.authenticate(user.username, "password123"):
                self.logger.info(f"Authentication successful for {user.username}")
            else:
                self.logger.error(f"Authentication failed for {user.username}")
    
    def demonstrate_user_management(self) -> None:
        """Demonstrate user management functionality."""
        # Get all users
        all_users = self.user_manager.get_all_users()
        self.logger.info(f"Total users: {len(all_users)}")
        
        # Find user by username
        user = self.user_manager.find_by_username("alice")
        if user:
            self.logger.info(f"Found user: {user.username} ({user.email})")
            
            # Update user
            user.age = 31
            self.user_manager.update_user(user)
            self.logger.info(f"Updated user age to {user.age}")
        
        # Get users by age range
        young_users = self.user_manager.get_users_by_age_range(20, 30)
        self.logger.info(f"Users aged 20-30: {len(young_users)}")


def load_config() -> AppConfig:
    """Load application configuration."""
    config = Config()
    return AppConfig(
        debug=config.get_bool("DEBUG", False),
        database_url=config.get_string("DATABASE_URL", "sqlite:///app.db"),
        log_level=config.get_string("LOG_LEVEL", "INFO")
    )


def main():
    """Main entry point."""
    try:
        config = load_config()
        app = Application(config)
        app.run()
    except KeyboardInterrupt:
        print("\nApplication interrupted by user")
        sys.exit(0)
    except Exception as e:
        print(f"Unexpected error: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main() 