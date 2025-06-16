"""API request handlers."""

import json
from typing import Dict, Any, List, Optional
from datetime import datetime

from ..models.user import User, UserManager
from ..services.auth import AuthService
from ..utils.logger import Logger


class BaseHandler:
    """Base class for all API handlers."""
    
    def __init__(self, user_manager: UserManager, auth_service: AuthService, logger: Logger):
        self.user_manager = user_manager
        self.auth_service = auth_service
        self.logger = logger
    
    def validate_request(self, data: Dict[str, Any], required_fields: List[str]) -> List[str]:
        """Validate that required fields are present."""
        errors = []
        for field in required_fields:
            if field not in data:
                errors.append(f"Missing required field: {field}")
        return errors
    
    def create_response(self, status: str, data: Any = None, message: str = "") -> Dict[str, Any]:
        """Create standardized response."""
        response = {
            "status": status,
            "timestamp": datetime.now().isoformat(),
            "message": message
        }
        if data is not None:
            response["data"] = data
        return response
    
    def handle_error(self, error: Exception, context: str = "") -> Dict[str, Any]:
        """Handle and log errors."""
        error_msg = f"{context}: {str(error)}" if context else str(error)
        self.logger.error(error_msg)
        return self.create_response("error", message=error_msg)


class UserHandler(BaseHandler):
    """Handler for user-related API endpoints."""
    
    def create_user(self, request_data: Dict[str, Any]) -> Dict[str, Any]:
        """Create a new user."""
        try:
            errors = self.validate_request(request_data, ["username", "email", "age"])
            if errors:
                return self.create_response("error", message="; ".join(errors))
            
            user = User(
                username=request_data["username"],
                email=request_data["email"],
                age=request_data["age"]
            )
            
            if self.user_manager.create_user(user):
                self.logger.info(f"Created user: {user.username}")
                return self.create_response(
                    "success", 
                    data=user.to_dict(),
                    message="User created successfully"
                )
            else:
                return self.create_response("error", message="Failed to create user")
                
        except Exception as e:
            return self.handle_error(e, "create_user")
    
    def get_user(self, user_id: str) -> Dict[str, Any]:
        """Get a user by ID."""
        try:
            user = self.user_manager.get_user(user_id)
            if user:
                return self.create_response(
                    "success",
                    data=user.to_dict(),
                    message="User retrieved successfully"
                )
            else:
                return self.create_response("error", message="User not found")
                
        except Exception as e:
            return self.handle_error(e, "get_user")
    
    def update_user(self, user_id: str, request_data: Dict[str, Any]) -> Dict[str, Any]:
        """Update an existing user."""
        try:
            user = self.user_manager.get_user(user_id)
            if not user:
                return self.create_response("error", message="User not found")
            
            # Update allowed fields
            if "email" in request_data:
                user.email = request_data["email"]
            if "age" in request_data:
                user.age = request_data["age"]
            if "is_active" in request_data:
                user.is_active = request_data["is_active"]
            
            if self.user_manager.update_user(user):
                self.logger.info(f"Updated user: {user.username}")
                return self.create_response(
                    "success",
                    data=user.to_dict(),
                    message="User updated successfully"
                )
            else:
                return self.create_response("error", message="Failed to update user")
                
        except Exception as e:
            return self.handle_error(e, "update_user")
    
    def delete_user(self, user_id: str) -> Dict[str, Any]:
        """Delete a user."""
        try:
            if self.user_manager.delete_user(user_id):
                self.logger.info(f"Deleted user: {user_id}")
                return self.create_response("success", message="User deleted successfully")
            else:
                return self.create_response("error", message="User not found or deletion failed")
                
        except Exception as e:
            return self.handle_error(e, "delete_user")
    
    def list_users(self, filters: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        """List users with optional filters."""
        try:
            if filters and "age_range" in filters:
                min_age = filters["age_range"].get("min", 0)
                max_age = filters["age_range"].get("max", 150)
                users = self.user_manager.get_users_by_age_range(min_age, max_age)
            elif filters and "active_only" in filters:
                users = self.user_manager.get_active_users()
            else:
                users = self.user_manager.get_all_users()
            
            return self.create_response(
                "success",
                data=[user.to_dict() for user in users],
                message=f"Retrieved {len(users)} users"
            )
            
        except Exception as e:
            return self.handle_error(e, "list_users")
    
    def search_users(self, query: str) -> Dict[str, Any]:
        """Search users by username or email."""
        try:
            users = self.user_manager.search_users(query)
            return self.create_response(
                "success",
                data=[user.to_dict() for user in users],
                message=f"Found {len(users)} users matching '{query}'"
            )
            
        except Exception as e:
            return self.handle_error(e, "search_users")
    
    def get_user_stats(self) -> Dict[str, Any]:
        """Get user statistics."""
        try:
            stats = self.user_manager.get_user_stats()
            return self.create_response(
                "success",
                data=stats,
                message="User statistics retrieved successfully"
            )
            
        except Exception as e:
            return self.handle_error(e, "get_user_stats")


class AuthHandler(BaseHandler):
    """Handler for authentication-related API endpoints."""
    
    def login(self, request_data: Dict[str, Any]) -> Dict[str, Any]:
        """Authenticate a user."""
        try:
            errors = self.validate_request(request_data, ["username", "password"])
            if errors:
                return self.create_response("error", message="; ".join(errors))
            
            username = request_data["username"]
            password = request_data["password"]
            
            if self.auth_service.authenticate(username, password):
                user = self.user_manager.find_by_username(username)
                token = self.auth_service.generate_token(username)
                
                self.logger.info(f"User authenticated: {username}")
                return self.create_response(
                    "success",
                    data={
                        "user": user.to_dict() if user else None,
                        "token": token
                    },
                    message="Authentication successful"
                )
            else:
                self.logger.warning(f"Authentication failed for: {username}")
                return self.create_response("error", message="Invalid credentials")
                
        except Exception as e:
            return self.handle_error(e, "login")
    
    def logout(self, token: str) -> Dict[str, Any]:
        """Log out a user."""
        try:
            if self.auth_service.invalidate_token(token):
                self.logger.info("User logged out successfully")
                return self.create_response("success", message="Logout successful")
            else:
                return self.create_response("error", message="Invalid token")
                
        except Exception as e:
            return self.handle_error(e, "logout")
    
    def register(self, request_data: Dict[str, Any]) -> Dict[str, Any]:
        """Register a new user."""
        try:
            errors = self.validate_request(request_data, ["username", "email", "age", "password"])
            if errors:
                return self.create_response("error", message="; ".join(errors))
            
            # Create user
            user = User(
                username=request_data["username"],
                email=request_data["email"],
                age=request_data["age"]
            )
            
            if self.user_manager.create_user(user):
                # Set password
                self.auth_service.set_password(user.username, request_data["password"])
                
                self.logger.info(f"User registered: {user.username}")
                return self.create_response(
                    "success",
                    data=user.to_dict(),
                    message="User registered successfully"
                )
            else:
                return self.create_response("error", message="Failed to register user")
                
        except Exception as e:
            return self.handle_error(e, "register")
    
    def change_password(self, username: str, request_data: Dict[str, Any]) -> Dict[str, Any]:
        """Change user password."""
        try:
            errors = self.validate_request(request_data, ["old_password", "new_password"])
            if errors:
                return self.create_response("error", message="; ".join(errors))
            
            old_password = request_data["old_password"]
            new_password = request_data["new_password"]
            
            # Verify old password
            if not self.auth_service.authenticate(username, old_password):
                return self.create_response("error", message="Invalid current password")
            
            # Set new password
            if self.auth_service.set_password(username, new_password):
                self.logger.info(f"Password changed for user: {username}")
                return self.create_response("success", message="Password changed successfully")
            else:
                return self.create_response("error", message="Failed to change password")
                
        except Exception as e:
            return self.handle_error(e, "change_password") 