"""API route definitions and setup."""

from typing import Dict, Any, Callable, List
from .handlers import UserHandler, AuthHandler
from .middleware import AuthMiddleware


class Router:
    """Simple router for API endpoints."""
    
    def __init__(self):
        self.routes: Dict[str, Dict[str, Callable]] = {}
        self.middleware: List[Callable] = []
    
    def add_route(self, method: str, path: str, handler: Callable):
        """Add a route to the router."""
        if path not in self.routes:
            self.routes[path] = {}
        self.routes[path][method] = handler
    
    def add_middleware(self, middleware: Callable):
        """Add middleware to the router."""
        self.middleware.append(middleware)
    
    def get_handler(self, method: str, path: str) -> Callable:
        """Get handler for a route."""
        if path in self.routes and method in self.routes[path]:
            return self.routes[path][method]
        return None
    
    def handle_request(self, method: str, path: str, data: Dict[str, Any] = None) -> Dict[str, Any]:
        """Handle a request through the router."""
        # Apply middleware
        for middleware_func in self.middleware:
            result = middleware_func(method, path, data)
            if result.get("status") == "error":
                return result
        
        # Get and execute handler
        handler = self.get_handler(method, path)
        if handler:
            return handler(data or {})
        else:
            return {
                "status": "error",
                "message": f"Route not found: {method} {path}"
            }


def setup_routes(user_handler: UserHandler, auth_handler: AuthHandler, auth_middleware: AuthMiddleware) -> Router:
    """Set up all API routes."""
    router = Router()
    
    # Add middleware
    router.add_middleware(auth_middleware.authenticate_request)
    
    # Authentication routes (public)
    router.add_route("POST", "/auth/login", auth_handler.login)
    router.add_route("POST", "/auth/register", auth_handler.register)
    router.add_route("POST", "/auth/logout", auth_handler.logout)
    
    # User management routes (protected)
    router.add_route("GET", "/users", user_handler.list_users)
    router.add_route("POST", "/users", user_handler.create_user)
    router.add_route("GET", "/users/stats", user_handler.get_user_stats)
    router.add_route("GET", "/users/search", user_handler.search_users)
    router.add_route("GET", "/users/{id}", user_handler.get_user)
    router.add_route("PUT", "/users/{id}", user_handler.update_user)
    router.add_route("DELETE", "/users/{id}", user_handler.delete_user)
    
    # Password management routes (protected)
    router.add_route("POST", "/auth/change-password", auth_handler.change_password)
    
    return router


def create_api_app(user_handler: UserHandler, auth_handler: AuthHandler) -> Router:
    """Create the complete API application."""
    auth_middleware = AuthMiddleware()
    return setup_routes(user_handler, auth_handler, auth_middleware)


# Route decorators for more complex patterns
def route(method: str, path: str):
    """Decorator for route handlers."""
    def decorator(func):
        func._route_method = method
        func._route_path = path
        return func
    return decorator


def requires_auth(func):
    """Decorator for routes that require authentication."""
    func._requires_auth = True
    return func


class APIController:
    """Base controller class with route definitions."""
    
    def __init__(self, user_handler: UserHandler, auth_handler: AuthHandler):
        self.user_handler = user_handler
        self.auth_handler = auth_handler
    
    @route("GET", "/health")
    def health_check(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Health check endpoint."""
        return {
            "status": "success",
            "message": "API is healthy",
            "data": {
                "service": "gcore-api",
                "version": "1.0.0"
            }
        }
    
    @route("GET", "/version")
    def get_version(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Get API version."""
        return {
            "status": "success",
            "data": {
                "version": "1.0.0",
                "build": "dev",
                "features": ["user_management", "authentication", "search"]
            }
        }
    
    @route("POST", "/users/bulk")
    @requires_auth
    def bulk_create_users(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Create multiple users at once."""
        try:
            users_data = data.get("users", [])
            if not users_data:
                return {
                    "status": "error",
                    "message": "No users data provided"
                }
            
            results = []
            for user_data in users_data:
                result = self.user_handler.create_user(user_data)
                results.append(result)
            
            successful = sum(1 for r in results if r["status"] == "success")
            
            return {
                "status": "success",
                "message": f"Created {successful}/{len(users_data)} users",
                "data": {
                    "total": len(users_data),
                    "successful": successful,
                    "failed": len(users_data) - successful,
                    "results": results
                }
            }
            
        except Exception as e:
            return {
                "status": "error",
                "message": f"Bulk creation failed: {str(e)}"
            }
    
    def get_routes(self) -> Dict[str, Any]:
        """Get all routes defined in this controller."""
        routes = {}
        for attr_name in dir(self):
            attr = getattr(self, attr_name)
            if hasattr(attr, '_route_method') and hasattr(attr, '_route_path'):
                route_key = f"{attr._route_method} {attr._route_path}"
                routes[route_key] = {
                    "method": attr._route_method,
                    "path": attr._route_path,
                    "handler": attr_name,
                    "requires_auth": hasattr(attr, '_requires_auth')
                }
        return routes 