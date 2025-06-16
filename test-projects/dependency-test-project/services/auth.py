"""
Authentication service for agent system.
"""

import asyncio
import hashlib
import secrets
from typing import Dict, Optional


class AuthProvider:
    """Provides authentication services."""
    
    def __init__(self):
        self._tokens: Dict[str, str] = {}  # token -> user_id
        self._users: Dict[str, str] = {}  # user_id -> hashed_token
        self._initialized = False
    
    async def initialize(self) -> None:
        """Initialize the auth provider."""
        self._initialized = True
        # Create some default tokens
        await self._create_default_tokens()
    
    async def validate_token(self, token: str) -> bool:
        """Validate an authentication token."""
        if not self._initialized:
            return False
        
        return token in self._tokens
    
    async def create_token(self, user_id: str) -> str:
        """Create a new authentication token for a user."""
        if not self._initialized:
            raise RuntimeError("AuthProvider not initialized")
        
        token = secrets.token_urlsafe(32)
        self._tokens[token] = user_id
        self._users[user_id] = self._hash_token(token)
        
        return token
    
    async def revoke_token(self, token: str) -> bool:
        """Revoke an authentication token."""
        if token in self._tokens:
            user_id = self._tokens[token]
            del self._tokens[token]
            if user_id in self._users:
                del self._users[user_id]
            return True
        return False
    
    async def health_check(self) -> Dict[str, str]:
        """Health check for auth provider."""
        return {
            "status": "healthy" if self._initialized else "not_initialized",
            "active_tokens": str(len(self._tokens))
        }
    
    def _hash_token(self, token: str) -> str:
        """Hash a token for storage."""
        return hashlib.sha256(token.encode()).hexdigest()
    
    async def _create_default_tokens(self) -> None:
        """Create some default tokens for testing."""
        await self.create_token("agent_system")
        await self.create_token("test_user") 