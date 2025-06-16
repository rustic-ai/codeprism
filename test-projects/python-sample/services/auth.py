"""Authentication service for user management."""

import hashlib
import secrets
import time
from typing import Optional, Dict, Any
from datetime import datetime, timedelta


class AuthService:
    """Handles user authentication and session management."""
    
    def __init__(self, database_service):
        self.db_service = database_service
        self.sessions: Dict[str, Dict[str, Any]] = {}
        self.failed_attempts: Dict[str, int] = {}
        self.lockout_duration = 300  # 5 minutes
        self.max_attempts = 5
        
    def hash_password(self, password: str, salt: Optional[str] = None) -> tuple[str, str]:
        """Hash a password with salt."""
        if salt is None:
            salt = secrets.token_hex(16)
        
        # Use PBKDF2 for password hashing
        password_hash = hashlib.pbkdf2_hmac(
            'sha256',
            password.encode('utf-8'),
            salt.encode('utf-8'),
            100000  # iterations
        )
        
        return password_hash.hex(), salt
    
    def verify_password(self, password: str, stored_hash: str, salt: str) -> bool:
        """Verify a password against stored hash."""
        password_hash, _ = self.hash_password(password, salt)
        return secrets.compare_digest(password_hash, stored_hash)
    
    def set_password(self, username: str, password: str) -> bool:
        """Set password for a user."""
        try:
            password_hash, salt = self.hash_password(password)
            return self.db_service.store_password(username, password_hash, salt)
        except Exception as e:
            print(f"Error setting password: {e}")
            return False
    
    def is_account_locked(self, username: str) -> bool:
        """Check if account is locked due to failed attempts."""
        if username not in self.failed_attempts:
            return False
        
        attempts = self.failed_attempts[username]
        if attempts >= self.max_attempts:
            # Check if lockout period has expired
            last_attempt_time = getattr(self, f"_last_attempt_{username}", 0)
            if time.time() - last_attempt_time > self.lockout_duration:
                # Reset failed attempts
                del self.failed_attempts[username]
                return False
            return True
        
        return False
    
    def record_failed_attempt(self, username: str) -> None:
        """Record a failed authentication attempt."""
        self.failed_attempts[username] = self.failed_attempts.get(username, 0) + 1
        setattr(self, f"_last_attempt_{username}", time.time())
    
    def reset_failed_attempts(self, username: str) -> None:
        """Reset failed attempts for a user."""
        if username in self.failed_attempts:
            del self.failed_attempts[username]
    
    def authenticate(self, username: str, password: str) -> bool:
        """Authenticate a user."""
        try:
            # Check if account is locked
            if self.is_account_locked(username):
                print(f"Account {username} is locked due to too many failed attempts")
                return False
            
            # Get stored password data
            password_data = self.db_service.get_password_data(username)
            if not password_data:
                self.record_failed_attempt(username)
                return False
            
            stored_hash = password_data.get("password_hash")
            salt = password_data.get("salt")
            
            if not stored_hash or not salt:
                self.record_failed_attempt(username)
                return False
            
            # Verify password
            if self.verify_password(password, stored_hash, salt):
                self.reset_failed_attempts(username)
                return True
            else:
                self.record_failed_attempt(username)
                return False
                
        except Exception as e:
            print(f"Authentication error: {e}")
            self.record_failed_attempt(username)
            return False
    
    def create_session(self, username: str) -> str:
        """Create a new session for authenticated user."""
        session_token = secrets.token_urlsafe(32)
        session_data = {
            "username": username,
            "created_at": datetime.now(),
            "last_activity": datetime.now(),
            "expires_at": datetime.now() + timedelta(hours=24)
        }
        
        self.sessions[session_token] = session_data
        return session_token
    
    def validate_session(self, session_token: str) -> Optional[str]:
        """Validate a session token and return username if valid."""
        if session_token not in self.sessions:
            return None
        
        session_data = self.sessions[session_token]
        
        # Check if session has expired
        if datetime.now() > session_data["expires_at"]:
            del self.sessions[session_token]
            return None
        
        # Update last activity
        session_data["last_activity"] = datetime.now()
        
        return session_data["username"]
    
    def logout(self, session_token: str) -> bool:
        """Logout user by invalidating session."""
        if session_token in self.sessions:
            del self.sessions[session_token]
            return True
        return False
    
    def extend_session(self, session_token: str, hours: int = 24) -> bool:
        """Extend session expiration time."""
        if session_token not in self.sessions:
            return False
        
        session_data = self.sessions[session_token]
        session_data["expires_at"] = datetime.now() + timedelta(hours=hours)
        return True
    
    def get_active_sessions(self) -> Dict[str, Dict[str, Any]]:
        """Get all active sessions."""
        now = datetime.now()
        active_sessions = {}
        
        # Clean up expired sessions
        expired_tokens = []
        for token, data in self.sessions.items():
            if now > data["expires_at"]:
                expired_tokens.append(token)
            else:
                active_sessions[token] = {
                    "username": data["username"],
                    "created_at": data["created_at"].isoformat(),
                    "last_activity": data["last_activity"].isoformat(),
                    "expires_at": data["expires_at"].isoformat()
                }
        
        # Remove expired sessions
        for token in expired_tokens:
            del self.sessions[token]
        
        return active_sessions
    
    def cleanup_expired_sessions(self) -> int:
        """Clean up expired sessions and return count removed."""
        now = datetime.now()
        expired_tokens = [
            token for token, data in self.sessions.items()
            if now > data["expires_at"]
        ]
        
        for token in expired_tokens:
            del self.sessions[token]
        
        return len(expired_tokens)
    
    def get_session_info(self, session_token: str) -> Optional[Dict[str, Any]]:
        """Get information about a specific session."""
        if session_token not in self.sessions:
            return None
        
        data = self.sessions[session_token]
        return {
            "username": data["username"],
            "created_at": data["created_at"].isoformat(),
            "last_activity": data["last_activity"].isoformat(),
            "expires_at": data["expires_at"].isoformat(),
            "is_expired": datetime.now() > data["expires_at"]
        }
    
    def change_password(self, username: str, old_password: str, new_password: str) -> bool:
        """Change user password."""
        # Verify old password first
        if not self.authenticate(username, old_password):
            return False
        
        # Set new password
        return self.set_password(username, new_password)
    
    def generate_password_reset_token(self, username: str) -> Optional[str]:
        """Generate a password reset token."""
        # Check if user exists
        if not self.db_service.get_user_by_username(username):
            return None
        
        reset_token = secrets.token_urlsafe(32)
        expiry = datetime.now() + timedelta(hours=1)  # 1 hour expiry
        
        # Store reset token (in real app, this would be in database)
        self.db_service.store_reset_token(username, reset_token, expiry)
        
        return reset_token
    
    def reset_password_with_token(self, reset_token: str, new_password: str) -> bool:
        """Reset password using a reset token."""
        username = self.db_service.validate_reset_token(reset_token)
        if not username:
            return False
        
        # Set new password
        success = self.set_password(username, new_password)
        
        if success:
            # Invalidate the reset token
            self.db_service.invalidate_reset_token(reset_token)
            # Clear any existing sessions for this user
            self._invalidate_user_sessions(username)
        
        return success
    
    def _invalidate_user_sessions(self, username: str) -> None:
        """Invalidate all sessions for a specific user."""
        tokens_to_remove = [
            token for token, data in self.sessions.items()
            if data["username"] == username
        ]
        
        for token in tokens_to_remove:
            del self.sessions[token]


def generate_secure_token(length: int = 32) -> str:
    """Generate a cryptographically secure random token."""
    return secrets.token_urlsafe(length)


def hash_data(data: str) -> str:
    """Hash arbitrary data using SHA-256."""
    return hashlib.sha256(data.encode('utf-8')).hexdigest()


def constant_time_compare(a: str, b: str) -> bool:
    """Compare two strings in constant time to prevent timing attacks."""
    return secrets.compare_digest(a, b) 