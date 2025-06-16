"""API module for web endpoints."""

from .handlers import UserHandler, AuthHandler
from .routes import setup_routes
from .middleware import AuthMiddleware

__all__ = ['UserHandler', 'AuthHandler', 'setup_routes', 'AuthMiddleware'] 