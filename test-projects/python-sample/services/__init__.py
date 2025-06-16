"""Service layer with advanced patterns and dependency injection."""

from .auth import AuthenticationService, AuthorizationService, JWTTokenService
from .cache import CacheService, RedisCacheService, MemoryCacheService
from .email import EmailService, SMTPEmailService, TemplateEmailService
from .notification import NotificationService, PushNotificationService, WebhookNotificationService
from .user import UserService, UserCreationService, UserManagementService
from .background import BackgroundTaskService, CeleryTaskService, TaskScheduler
from .analytics import AnalyticsService, EventTrackingService, MetricsCollectionService

__all__ = [
    # Authentication & Authorization
    "AuthenticationService",
    "AuthorizationService", 
    "JWTTokenService",
    
    # Caching
    "CacheService",
    "RedisCacheService",
    "MemoryCacheService",
    
    # Communication
    "EmailService",
    "SMTPEmailService",
    "TemplateEmailService",
    "NotificationService",
    "PushNotificationService", 
    "WebhookNotificationService",
    
    # User Management
    "UserService",
    "UserCreationService",
    "UserManagementService",
    
    # Background Processing
    "BackgroundTaskService",
    "CeleryTaskService",
    "TaskScheduler",
    
    # Analytics
    "AnalyticsService",
    "EventTrackingService",
    "MetricsCollectionService",
] 