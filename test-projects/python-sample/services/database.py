"""Database service for data persistence."""

import json
import os
from typing import List, Optional, Dict, Any
from datetime import datetime
from models.user import User


class DatabaseService:
    """Mock database service for demonstration purposes."""
    
    def __init__(self, database_url: str):
        self.database_url = database_url
        self.connected = False
        self._users: Dict[str, User] = {}
        self._passwords: Dict[str, Dict[str, str]] = {}
        self._reset_tokens: Dict[str, Dict[str, Any]] = {}
        
    def connect(self) -> bool:
        """Connect to the database."""
        try:
            # In a real implementation, this would connect to actual database
            print(f"Connecting to database: {self.database_url}")
            self.connected = True
            return True
        except Exception as e:
            print(f"Database connection failed: {e}")
            return False
    
    def disconnect(self) -> None:
        """Disconnect from the database."""
        self.connected = False
        print("Disconnected from database")
    
    def create_tables(self) -> bool:
        """Create necessary database tables."""
        if not self.connected:
            raise RuntimeError("Database not connected")
        
        try:
            # In a real implementation, this would create actual tables
            print("Creating database tables...")
            print("- users table created")
            print("- passwords table created")
            print("- reset_tokens table created")
            return True
        except Exception as e:
            print(f"Failed to create tables: {e}")
            return False
    
    def save_user(self, user: User) -> bool:
        """Save a user to the database."""
        if not self.connected:
            raise RuntimeError("Database not connected")
        
        try:
            self._users[user.user_id] = user
            print(f"User {user.username} saved to database")
            return True
        except Exception as e:
            print(f"Failed to save user: {e}")
            return False
    
    def get_user_by_id(self, user_id: str) -> Optional[User]:
        """Get user by ID."""
        if not self.connected:
            raise RuntimeError("Database not connected")
        
        return self._users.get(user_id)
    
    def get_user_by_username(self, username: str) -> Optional[User]:
        """Get user by username."""
        if not self.connected:
            raise RuntimeError("Database not connected")
        
        for user in self._users.values():
            if user.username == username:
                return user
        return None
    
    def get_user_by_email(self, email: str) -> Optional[User]:
        """Get user by email."""
        if not self.connected:
            raise RuntimeError("Database not connected")
        
        for user in self._users.values():
            if user.email == email:
                return user
        return None
    
    def update_user(self, user: User) -> bool:
        """Update an existing user."""
        if not self.connected:
            raise RuntimeError("Database not connected")
        
        try:
            if user.user_id in self._users:
                self._users[user.user_id] = user
                print(f"User {user.username} updated in database")
                return True
            else:
                print(f"User {user.user_id} not found for update")
                return False
        except Exception as e:
            print(f"Failed to update user: {e}")
            return False
    
    def delete_user(self, user_id: str) -> bool:
        """Delete a user from the database."""
        if not self.connected:
            raise RuntimeError("Database not connected")
        
        try:
            if user_id in self._users:
                user = self._users[user_id]
                del self._users[user_id]
                
                # Also remove password data
                if user.username in self._passwords:
                    del self._passwords[user.username]
                
                print(f"User {user.username} deleted from database")
                return True
            else:
                print(f"User {user_id} not found for deletion")
                return False
        except Exception as e:
            print(f"Failed to delete user: {e}")
            return False
    
    def get_all_users(self) -> List[User]:
        """Get all users from the database."""
        if not self.connected:
            raise RuntimeError("Database not connected")
        
        return list(self._users.values())
    
    def store_password(self, username: str, password_hash: str, salt: str) -> bool:
        """Store password hash and salt for a user."""
        if not self.connected:
            raise RuntimeError("Database not connected")
        
        try:
            self._passwords[username] = {
                "password_hash": password_hash,
                "salt": salt,
                "updated_at": datetime.now().isoformat()
            }
            return True
        except Exception as e:
            print(f"Failed to store password: {e}")
            return False
    
    def get_password_data(self, username: str) -> Optional[Dict[str, str]]:
        """Get password data for a user."""
        if not self.connected:
            raise RuntimeError("Database not connected")
        
        return self._passwords.get(username)
    
    def store_reset_token(self, username: str, token: str, expiry: datetime) -> bool:
        """Store a password reset token."""
        if not self.connected:
            raise RuntimeError("Database not connected")
        
        try:
            self._reset_tokens[token] = {
                "username": username,
                "expiry": expiry.isoformat(),
                "created_at": datetime.now().isoformat()
            }
            return True
        except Exception as e:
            print(f"Failed to store reset token: {e}")
            return False
    
    def validate_reset_token(self, token: str) -> Optional[str]:
        """Validate a reset token and return username if valid."""
        if not self.connected:
            raise RuntimeError("Database not connected")
        
        token_data = self._reset_tokens.get(token)
        if not token_data:
            return None
        
        expiry = datetime.fromisoformat(token_data["expiry"])
        if datetime.now() > expiry:
            # Token expired, remove it
            del self._reset_tokens[token]
            return None
        
        return token_data["username"]
    
    def invalidate_reset_token(self, token: str) -> bool:
        """Invalidate a reset token."""
        if not self.connected:
            raise RuntimeError("Database not connected")
        
        if token in self._reset_tokens:
            del self._reset_tokens[token]
            return True
        return False
    
    def execute_query(self, query: str, params: Optional[Dict[str, Any]] = None) -> List[Dict[str, Any]]:
        """Execute a raw SQL query (mock implementation)."""
        if not self.connected:
            raise RuntimeError("Database not connected")
        
        # Mock implementation - in real app this would execute actual SQL
        print(f"Executing query: {query}")
        if params:
            print(f"Parameters: {params}")
        
        # Return mock results based on query type
        if "SELECT" in query.upper():
            if "users" in query.lower():
                return [user.to_dict() for user in self._users.values()]
            elif "passwords" in query.lower():
                return [{"username": k, **v} for k, v in self._passwords.items()]
        
        return []
    
    def get_database_stats(self) -> Dict[str, Any]:
        """Get database statistics."""
        if not self.connected:
            raise RuntimeError("Database not connected")
        
        return {
            "total_users": len(self._users),
            "total_passwords": len(self._passwords),
            "active_reset_tokens": len(self._reset_tokens),
            "database_url": self.database_url,
            "connected": self.connected
        }
    
    def backup_data(self, backup_path: str) -> bool:
        """Backup database data to a file."""
        if not self.connected:
            raise RuntimeError("Database not connected")
        
        try:
            backup_data = {
                "users": {uid: user.to_dict() for uid, user in self._users.items()},
                "passwords": self._passwords,
                "reset_tokens": self._reset_tokens,
                "backup_timestamp": datetime.now().isoformat()
            }
            
            with open(backup_path, 'w') as f:
                json.dump(backup_data, f, indent=2)
            
            print(f"Database backed up to {backup_path}")
            return True
        except Exception as e:
            print(f"Backup failed: {e}")
            return False
    
    def restore_data(self, backup_path: str) -> bool:
        """Restore database data from a backup file."""
        if not self.connected:
            raise RuntimeError("Database not connected")
        
        try:
            if not os.path.exists(backup_path):
                print(f"Backup file not found: {backup_path}")
                return False
            
            with open(backup_path, 'r') as f:
                backup_data = json.load(f)
            
            # Restore users
            self._users = {}
            for uid, user_data in backup_data.get("users", {}).items():
                user = User.from_dict(user_data)
                self._users[uid] = user
            
            # Restore passwords and reset tokens
            self._passwords = backup_data.get("passwords", {})
            self._reset_tokens = backup_data.get("reset_tokens", {})
            
            print(f"Database restored from {backup_path}")
            return True
        except Exception as e:
            print(f"Restore failed: {e}")
            return False
    
    def clear_all_data(self) -> bool:
        """Clear all data from the database (for testing)."""
        if not self.connected:
            raise RuntimeError("Database not connected")
        
        try:
            self._users.clear()
            self._passwords.clear()
            self._reset_tokens.clear()
            print("All database data cleared")
            return True
        except Exception as e:
            print(f"Failed to clear data: {e}")
            return False
    
    def health_check(self) -> Dict[str, Any]:
        """Perform a health check on the database."""
        return {
            "status": "healthy" if self.connected else "disconnected",
            "connected": self.connected,
            "database_url": self.database_url,
            "data_counts": {
                "users": len(self._users),
                "passwords": len(self._passwords),
                "reset_tokens": len(self._reset_tokens)
            },
            "timestamp": datetime.now().isoformat()
        }


class DatabaseConnectionPool:
    """Mock database connection pool."""
    
    def __init__(self, database_url: str, pool_size: int = 5):
        self.database_url = database_url
        self.pool_size = pool_size
        self.connections: List[DatabaseService] = []
        self.active_connections = 0
        
    def get_connection(self) -> DatabaseService:
        """Get a connection from the pool."""
        if self.active_connections < self.pool_size:
            conn = DatabaseService(self.database_url)
            conn.connect()
            self.connections.append(conn)
            self.active_connections += 1
            return conn
        else:
            # In real implementation, this would wait for available connection
            return self.connections[0]
    
    def release_connection(self, connection: DatabaseService) -> None:
        """Release a connection back to the pool."""
        # In real implementation, this would return connection to pool
        pass
    
    def close_all(self) -> None:
        """Close all connections in the pool."""
        for conn in self.connections:
            conn.disconnect()
        self.connections.clear()
        self.active_connections = 0


def create_database_service(database_url: str) -> DatabaseService:
    """Factory function to create a database service."""
    return DatabaseService(database_url) 