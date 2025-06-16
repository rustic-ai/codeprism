"""API middleware for authentication and request processing."""

import re
from typing import Dict, Any, Optional, List
from datetime import datetime, timedelta


class AuthMiddleware:
    """Middleware for handling authentication."""
    
    def __init__(self):
        self.public_routes = [
            "/auth/login",
            "/auth/register", 
            "/health",
            "/version"
        ]
        self.active_tokens: Dict[str, Dict[str, Any]] = {}
        self.token_expiry = timedelta(hours=24)
    
    def authenticate_request(self, method: str, path: str, data: Dict[str, Any]) -> Dict[str, Any]:
        """Authenticate incoming requests."""
        # Skip authentication for public routes
        if self.is_public_route(path):
            return {"status": "success", "message": "Public route"}
        
        # Check for authorization token
        token = self.extract_token(data)
        if not token:
            return {
                "status": "error",
                "message": "Authentication required - no token provided"
            }
        
        # Validate token
        if not self.validate_token(token):
            return {
                "status": "error", 
                "message": "Invalid or expired token"
            }
        
        return {"status": "success", "message": "Authenticated"}
    
    def is_public_route(self, path: str) -> bool:
        """Check if a route is public (doesn't require authentication)."""
        return path in self.public_routes
    
    def extract_token(self, data: Dict[str, Any]) -> Optional[str]:
        """Extract authentication token from request data."""
        # Look for token in various places
        if "authorization" in data:
            auth_header = data["authorization"]
            if auth_header.startswith("Bearer "):
                return auth_header[7:]  # Remove "Bearer " prefix
        
        if "token" in data:
            return data["token"]
        
        return None
    
    def validate_token(self, token: str) -> bool:
        """Validate an authentication token."""
        if token not in self.active_tokens:
            return False
        
        token_data = self.active_tokens[token]
        
        # Check if token has expired
        issued_at = datetime.fromisoformat(token_data["issued_at"])
        if datetime.now() - issued_at > self.token_expiry:
            # Remove expired token
            del self.active_tokens[token]
            return False
        
        # Update last used time
        token_data["last_used"] = datetime.now().isoformat()
        return True
    
    def generate_token(self, username: str) -> str:
        """Generate a new authentication token."""
        import secrets
        token = secrets.token_urlsafe(32)
        
        self.active_tokens[token] = {
            "username": username,
            "issued_at": datetime.now().isoformat(),
            "last_used": datetime.now().isoformat()
        }
        
        return token
    
    def invalidate_token(self, token: str) -> bool:
        """Invalidate an authentication token."""
        if token in self.active_tokens:
            del self.active_tokens[token]
            return True
        return False
    
    def get_user_from_token(self, token: str) -> Optional[str]:
        """Get username associated with a token."""
        if token in self.active_tokens:
            return self.active_tokens[token]["username"]
        return None
    
    def cleanup_expired_tokens(self) -> int:
        """Remove expired tokens and return count of removed tokens."""
        now = datetime.now()
        expired_tokens = []
        
        for token, token_data in self.active_tokens.items():
            issued_at = datetime.fromisoformat(token_data["issued_at"])
            if now - issued_at > self.token_expiry:
                expired_tokens.append(token)
        
        for token in expired_tokens:
            del self.active_tokens[token]
        
        return len(expired_tokens)


class RateLimitMiddleware:
    """Middleware for rate limiting requests."""
    
    def __init__(self, requests_per_minute: int = 60):
        self.requests_per_minute = requests_per_minute
        self.request_counts: Dict[str, List[datetime]] = {}
    
    def check_rate_limit(self, identifier: str) -> Dict[str, Any]:
        """Check if request should be rate limited."""
        now = datetime.now()
        minute_ago = now - timedelta(minutes=1)
        
        # Initialize or clean up old requests for this identifier
        if identifier not in self.request_counts:
            self.request_counts[identifier] = []
        else:
            # Remove requests older than 1 minute
            self.request_counts[identifier] = [
                req_time for req_time in self.request_counts[identifier]
                if req_time > minute_ago
            ]
        
        # Check if rate limit exceeded
        if len(self.request_counts[identifier]) >= self.requests_per_minute:
            return {
                "status": "error",
                "message": f"Rate limit exceeded: {self.requests_per_minute} requests per minute"
            }
        
        # Add current request
        self.request_counts[identifier].append(now)
        
        return {"status": "success", "message": "Rate limit OK"}
    
    def get_rate_limit_info(self, identifier: str) -> Dict[str, Any]:
        """Get rate limit information for an identifier."""
        now = datetime.now()
        minute_ago = now - timedelta(minutes=1)
        
        if identifier not in self.request_counts:
            return {
                "requests_made": 0,
                "requests_remaining": self.requests_per_minute,
                "reset_time": (now + timedelta(minutes=1)).isoformat()
            }
        
        # Count requests in last minute
        recent_requests = [
            req_time for req_time in self.request_counts[identifier]
            if req_time > minute_ago
        ]
        
        requests_made = len(recent_requests)
        requests_remaining = max(0, self.requests_per_minute - requests_made)
        
        # Find when the oldest request will expire
        reset_time = now + timedelta(minutes=1)
        if recent_requests:
            oldest_request = min(recent_requests)
            reset_time = oldest_request + timedelta(minutes=1)
        
        return {
            "requests_made": requests_made,
            "requests_remaining": requests_remaining,
            "reset_time": reset_time.isoformat()
        }


class LoggingMiddleware:
    """Middleware for logging requests."""
    
    def __init__(self, logger):
        self.logger = logger
        self.request_count = 0
    
    def log_request(self, method: str, path: str, data: Dict[str, Any]) -> Dict[str, Any]:
        """Log incoming requests."""
        self.request_count += 1
        
        # Sanitize sensitive data for logging
        safe_data = self.sanitize_data(data)
        
        self.logger.info(f"Request #{self.request_count}: {method} {path}")
        self.logger.debug(f"Request data: {safe_data}")
        
        return {"status": "success", "message": "Request logged"}
    
    def sanitize_data(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Remove sensitive information from data for logging."""
        sensitive_fields = ["password", "token", "authorization", "secret"]
        
        sanitized = {}
        for key, value in data.items():
            if key.lower() in sensitive_fields:
                sanitized[key] = "[REDACTED]"
            elif isinstance(value, dict):
                sanitized[key] = self.sanitize_data(value)
            else:
                sanitized[key] = value
        
        return sanitized
    
    def get_request_stats(self) -> Dict[str, Any]:
        """Get request statistics."""
        return {
            "total_requests": self.request_count,
            "service_start_time": datetime.now().isoformat()  # In real app, this would be actual start time
        }


class ValidationMiddleware:
    """Middleware for request validation."""
    
    def __init__(self):
        self.validation_rules: Dict[str, Dict[str, Any]] = {
            "/users": {
                "POST": {
                    "required_fields": ["username", "email", "age"],
                    "field_types": {
                        "username": str,
                        "email": str,
                        "age": int
                    }
                }
            },
            "/auth/login": {
                "POST": {
                    "required_fields": ["username", "password"],
                    "field_types": {
                        "username": str,
                        "password": str
                    }
                }
            }
        }
    
    def validate_request(self, method: str, path: str, data: Dict[str, Any]) -> Dict[str, Any]:
        """Validate request data according to defined rules."""
        if path not in self.validation_rules:
            return {"status": "success", "message": "No validation rules for this endpoint"}
        
        if method not in self.validation_rules[path]:
            return {"status": "success", "message": "No validation rules for this method"}
        
        rules = self.validation_rules[path][method]
        errors = []
        
        # Check required fields
        for field in rules.get("required_fields", []):
            if field not in data:
                errors.append(f"Missing required field: {field}")
        
        # Check field types
        for field, expected_type in rules.get("field_types", {}).items():
            if field in data and not isinstance(data[field], expected_type):
                errors.append(f"Field '{field}' must be of type {expected_type.__name__}")
        
        # Check email format
        if "email" in data:
            if not self.is_valid_email(data["email"]):
                errors.append("Invalid email format")
        
        if errors:
            return {
                "status": "error",
                "message": "; ".join(errors)
            }
        
        return {"status": "success", "message": "Validation passed"}
    
    def is_valid_email(self, email: str) -> bool:
        """Validate email format."""
        pattern = r'^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$'
        return re.match(pattern, email) is not None 