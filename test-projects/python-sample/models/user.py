#!/usr/bin/env python3
"""
User models and management classes.
This module demonstrates inheritance hierarchies, decorators, and design patterns.
"""

import hashlib
import secrets
from datetime import datetime, timedelta
from enum import Enum
from typing import Any, Dict, List, Optional, Set
from uuid import UUID, uuid4
from abc import ABC, abstractmethod
from dataclasses import dataclass, field
import logging
from functools import wraps

from pydantic import Field, validator, root_validator
from sqlalchemy import Column, String, Boolean, DateTime, Integer, Text
from sqlalchemy.dialects.postgresql import UUID as PG_UUID, JSONB

from core import (
    BaseEntity,
    DomainEvent,
    BusinessLogicError,
    ValidationError,
    NotFoundError,
    AuthenticationError,
)
from core.types import JsonDict

# Type variables for generic patterns
T = TypeVar('T')
U = TypeVar('U')

# Enums for testing
class UserRole(Enum):
    """User role enumeration."""
    ADMIN = "admin"
    USER = "user"
    GUEST = "guest"
    MODERATOR = "moderator"

class UserStatus(Enum):
    """User status enumeration."""
    ACTIVE = "active"
    INACTIVE = "inactive"
    SUSPENDED = "suspended"
    PENDING = "pending"

# Decorator examples for testing
def audit_action(action: str):
    """Decorator to audit user actions."""
    def decorator(func):
        @wraps(func)
        def wrapper(*args, **kwargs):
            logger = logging.getLogger(__name__)
            logger.info(f"Auditing action: {action}")
            result = func(*args, **kwargs)
            logger.info(f"Action {action} completed")
            return result
        return wrapper
    return decorator

def validate_permissions(required_role: UserRole):
    """Decorator to validate user permissions."""
    def decorator(func):
        @wraps(func)
        def wrapper(self, *args, **kwargs):
            if hasattr(self, 'current_user') and self.current_user:
                if self.current_user.role.value >= required_role.value:
                    return func(self, *args, **kwargs)
                else:
                    raise PermissionError(f"Insufficient permissions. Required: {required_role}")
            raise CustomAuthenticationError("No authenticated user")
        return wrapper
    return decorator

def cache_result(cache_duration: int = 300):
    """Decorator to cache method results."""
    def decorator(func):
        cache = {}
        @wraps(func)
        def wrapper(*args, **kwargs):
            cache_key = str(args) + str(kwargs)
            if cache_key in cache:
                return cache[cache_key]
            result = func(*args, **kwargs)
            cache[cache_key] = result
            return result
        return wrapper
    return decorator

# Custom exceptions for inheritance hierarchy
class UserError(Exception):
    """Base user exception."""
    pass

class CustomAuthenticationError(UserError):
    """Authentication related errors."""
    pass

class AuthorizationError(UserError):
    """Authorization related errors."""
    pass

class CustomValidationError(UserError):
    """Validation related errors."""
    pass

# Abstract base classes for inheritance hierarchy
class Entity(ABC):
    """Abstract base entity class."""
    
    def __init__(self, id: Optional[int] = None):
        self.id = id
        self.created_at = datetime.now()
        self.updated_at = datetime.now()
    
    @abstractmethod
    def validate(self) -> bool:
        """Validate entity data."""
        pass
    
    @abstractmethod
    def to_dict(self) -> Dict[str, Any]:
        """Convert entity to dictionary."""
        pass

class Auditable(ABC):
    """Mixin for auditable entities."""
    
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.audit_log: List[Dict[str, Any]] = []
    
    def add_audit_entry(self, action: str, details: Dict[str, Any]):
        """Add an audit log entry."""
        self.audit_log.append({
            'timestamp': datetime.now(),
            'action': action,
            'details': details
        })

class Timestamped:
    """Mixin for timestamped entities."""
    
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.created_at = datetime.now()
        self.updated_at = datetime.now()
    
    def touch(self):
        """Update the timestamp."""
        self.updated_at = datetime.now()

# Main User class demonstrating multiple inheritance
@dataclass
class User(Entity, Auditable, Timestamped):
    """User model with inheritance hierarchy."""
    username: str = ""
    email: str = ""
    age: int = 0
    role: UserRole = UserRole.USER
    status: UserStatus = UserStatus.PENDING
    metadata: Dict[str, Any] = field(default_factory=dict)
    preferences: Dict[str, Any] = field(default_factory=dict)
    
    def __post_init__(self):
        """Initialize after dataclass creation."""
        Entity.__init__(self, self.id if hasattr(self, 'id') else None)
        Auditable.__init__(self)
        Timestamped.__init__(self)
        self.add_audit_entry("created", {"username": self.username})
    
    def validate(self) -> bool:
        """Validate user data."""
        if not self.username or len(self.username) < 3:
            raise CustomValidationError("Username must be at least 3 characters")
        if not self.email or '@' not in self.email:
            raise CustomValidationError("Valid email required")
        if self.age < 0 or self.age > 150:
            raise CustomValidationError("Age must be between 0 and 150")
        return True
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert user to dictionary."""
        return {
            'id': self.id,
            'username': self.username,
            'email': self.email,
            'age': self.age,
            'role': self.role.value,
            'status': self.status.value,
            'created_at': self.created_at.isoformat(),
            'updated_at': self.updated_at.isoformat(),
            'metadata': self.metadata,
            'preferences': self.preferences
        }
    
    @audit_action("profile_update")
    def update_profile(self, **kwargs):
        """Update user profile."""
        for key, value in kwargs.items():
            if hasattr(self, key):
                setattr(self, key, value)
        self.touch()
        self.add_audit_entry("profile_updated", kwargs)
    
    @cache_result(cache_duration=600)
    def get_permissions(self) -> List[str]:
        """Get user permissions based on role."""
        permission_map = {
            UserRole.ADMIN: ["read", "write", "delete", "admin"],
            UserRole.MODERATOR: ["read", "write", "moderate"],
            UserRole.USER: ["read", "write"],
            UserRole.GUEST: ["read"]
        }
        return permission_map.get(self.role, [])

# Specialized user types (single table inheritance pattern)
class AdminUser(User):
    """Administrator user with additional capabilities."""
    
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.role = UserRole.ADMIN
        self.admin_permissions: List[str] = ["system_config", "user_management"]
    
    @validate_permissions(UserRole.ADMIN)
    def manage_users(self, action: str, target_user: 'User'):
        """Manage other users (admin only)."""
        self.add_audit_entry("user_management", {
            "action": action,
            "target_user": target_user.username
        })
        # Implementation would go here
        pass
    
    def grant_permissions(self, user: 'User', permissions: List[str]):
        """Grant permissions to a user."""
        # Implementation would go here
        pass

class ModeratorUser(User):
    """Moderator user with content management capabilities."""
    
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.role = UserRole.MODERATOR
        self.moderation_permissions: List[str] = ["content_review", "user_warnings"]
    
    @validate_permissions(UserRole.MODERATOR)
    def moderate_content(self, content_id: str, action: str):
        """Moderate user content."""
        self.add_audit_entry("content_moderation", {
            "content_id": content_id,
            "action": action
        })

# Generic repository pattern
class Repository(Generic[T], ABC):
    """Generic repository interface."""
    
    @abstractmethod
    def find_by_id(self, id: int) -> Optional[T]:
        """Find entity by ID."""
        pass
    
    @abstractmethod
    def find_all(self) -> List[T]:
        """Find all entities."""
        pass
    
    @abstractmethod
    def save(self, entity: T) -> T:
        """Save entity."""
        pass
    
    @abstractmethod
    def delete(self, entity: T) -> bool:
        """Delete entity."""
        pass

# Observer pattern for user events
class UserObserver(ABC):
    """Observer interface for user events."""
    
    @abstractmethod
    def on_user_created(self, user: User):
        """Handle user creation event."""
        pass
    
    @abstractmethod
    def on_user_updated(self, user: User):
        """Handle user update event."""
        pass

class EmailNotificationObserver(UserObserver):
    """Email notification observer."""
    
    def on_user_created(self, user: User):
        """Send welcome email."""
        print(f"Sending welcome email to {user.email}")
    
    def on_user_updated(self, user: User):
        """Send update notification."""
        print(f"Sending update notification to {user.email}")

class AuditLogObserver(UserObserver):
    """Audit log observer."""
    
    def on_user_created(self, user: User):
        """Log user creation."""
        print(f"User created: {user.username}")
    
    def on_user_updated(self, user: User):
        """Log user update."""
        print(f"User updated: {user.username}")

# Singleton pattern for user manager
class UserManager:
    """User manager with repository pattern and observer support."""
    
    _instance = None
    _initialized = False
    
    def __new__(cls, db_service=None):
        if cls._instance is None:
            cls._instance = super(UserManager, cls).__new__(cls)
        return cls._instance
    
    def __init__(self, db_service=None):
        if not self._initialized:
            self.db_service = db_service
            self.users: List[User] = []
            self.observers: List[UserObserver] = []
            self._user_cache: Dict[str, User] = {}
            self._initialized = True
    
    def add_observer(self, observer: UserObserver):
        """Add user event observer."""
        self.observers.append(observer)
    
    def remove_observer(self, observer: UserObserver):
        """Remove user event observer."""
        if observer in self.observers:
            self.observers.remove(observer)
    
    def _notify_created(self, user: User):
        """Notify observers of user creation."""
        for observer in self.observers:
            observer.on_user_created(user)
    
    def _notify_updated(self, user: User):
        """Notify observers of user update."""
        for observer in self.observers:
            observer.on_user_updated(user)
    
    @audit_action("user_creation")
    def create_user(self, user: User) -> User:
        """Create a new user."""
        user.validate()
        self.users.append(user)
        self._user_cache[user.username] = user
        self._notify_created(user)
        return user
    
    @cache_result(cache_duration=300)
    def find_by_username(self, username: str) -> Optional[User]:
        """Find user by username."""
        return self._user_cache.get(username)
    
    def get_all_users(self) -> List[User]:
        """Get all users."""
        return self.users.copy()
    
    @audit_action("user_update")
    def update_user(self, user: User) -> User:
        """Update an existing user."""
        user.validate()
        user.touch()
        self._notify_updated(user)
        return user
    
    def get_users_by_age_range(self, min_age: int, max_age: int) -> List[User]:
        """Get users within age range."""
        return [user for user in self.users if min_age <= user.age <= max_age]
    
    def get_users_by_role(self, role: UserRole) -> List[User]:
        """Get users by role."""
        return [user for user in self.users if user.role == role]
    
    def get_active_users(self) -> List[User]:
        """Get active users."""
        return [user for user in self.users if user.status == UserStatus.ACTIVE]

# Builder pattern for complex user creation
class UserBuilder:
    """Builder pattern for creating complex user objects."""
    
    def __init__(self):
        self.reset()
    
    def reset(self):
        """Reset builder state."""
        self._user = User()
        return self
    
    def with_username(self, username: str):
        """Set username."""
        self._user.username = username
        return self
    
    def with_email(self, email: str):
        """Set email."""
        self._user.email = email
        return self
    
    def with_age(self, age: int):
        """Set age."""
        self._user.age = age
        return self
    
    def with_role(self, role: UserRole):
        """Set role."""
        self._user.role = role
        return self
    
    def with_status(self, status: UserStatus):
        """Set status."""
        self._user.status = status
        return self
    
    def with_metadata(self, key: str, value: Any):
        """Add metadata."""
        self._user.metadata[key] = value
        return self
    
    def with_preference(self, key: str, value: Any):
        """Add preference."""
        self._user.preferences[key] = value
        return self
    
    def build(self) -> User:
        """Build the user object."""
        user = self._user
        self.reset()
        return user

# Factory pattern for creating specialized users
class UserFactory:
    """Factory for creating different types of users."""
    
    @staticmethod
    def create_admin(username: str, email: str) -> AdminUser:
        """Create an admin user."""
        return AdminUser(username=username, email=email, role=UserRole.ADMIN)
    
    @staticmethod
    def create_moderator(username: str, email: str) -> ModeratorUser:
        """Create a moderator user."""
        return ModeratorUser(username=username, email=email, role=UserRole.MODERATOR)
    
    @staticmethod
    def create_user(username: str, email: str, age: int = 18) -> User:
        """Create a regular user."""
        return User(username=username, email=email, age=age, role=UserRole.USER)
    
    @staticmethod
    def create_guest(username: str) -> User:
        """Create a guest user."""
        return User(username=username, email="", age=0, role=UserRole.GUEST)

# Strategy pattern for user validation
class ValidationStrategy(ABC):
    """Abstract validation strategy."""
    
    @abstractmethod
    def validate(self, user: User) -> bool:
        """Validate user."""
        pass

class BasicValidationStrategy(ValidationStrategy):
    """Basic validation strategy."""
    
    def validate(self, user: User) -> bool:
        """Basic validation."""
        return len(user.username) >= 3 and '@' in user.email

class StrictValidationStrategy(ValidationStrategy):
    """Strict validation strategy."""
    
    def validate(self, user: User) -> bool:
        """Strict validation."""
        basic_valid = BasicValidationStrategy().validate(user)
        return (basic_valid and 
                len(user.username) >= 5 and
                user.age >= 13 and
                '.' in user.email)

class UserValidator:
    """User validator using strategy pattern."""
    
    def __init__(self, strategy: ValidationStrategy):
        self.strategy = strategy
    
    def set_strategy(self, strategy: ValidationStrategy):
        """Set validation strategy."""
        self.strategy = strategy
    
    def validate(self, user: User) -> bool:
        """Validate user using current strategy."""
        return self.strategy.validate(user)

# Domain Events
class UserCreatedEvent(DomainEvent):
    """Event raised when a user is created."""
    
    _aggregate_type = "User"
    
    username: str
    email: str
    role: UserRole
    

class UserUpdatedEvent(DomainEvent):
    """Event raised when a user is updated."""
    
    _aggregate_type = "User"
    
    changes: JsonDict


class UserDeactivatedEvent(DomainEvent):
    """Event raised when a user is deactivated."""
    
    _aggregate_type = "User"
    
    reason: Optional[str] = None


class UserPasswordChangedEvent(DomainEvent):
    """Event raised when a user's password is changed."""
    
    _aggregate_type = "User"
    
    changed_by: UUID  # User who made the change (could be admin)


class UserRoleChangedEvent(DomainEvent):
    """Event raised when a user's role is changed."""
    
    _aggregate_type = "User"
    
    old_role: UserRole
    new_role: UserRole
    changed_by: UUID


class User(BaseEntity):
    """Enhanced User aggregate root with comprehensive functionality."""
    
    # Core identity fields
    username: str = Field(min_length=3, max_length=50, description="Unique username")
    email: str = Field(description="User email address")
    password_hash: str = Field(description="Hashed password")
    
    # Status and role
    status: UserStatus = Field(default=UserStatus.PENDING, description="User status")
    role: UserRole = Field(default=UserRole.USER, description="User role")
    
    # Authentication fields
    email_verified: bool = Field(default=False, description="Email verification status")
    email_verification_token: Optional[str] = Field(default=None, description="Email verification token")
    password_reset_token: Optional[str] = Field(default=None, description="Password reset token")
    password_reset_expires: Optional[datetime] = Field(default=None, description="Password reset expiration")
    
    # Security fields
    failed_login_attempts: int = Field(default=0, description="Failed login attempts")
    locked_until: Optional[datetime] = Field(default=None, description="Account lock expiration")
    last_login: Optional[datetime] = Field(default=None, description="Last successful login")
    
    # Profile and preferences
    profile: Optional[UserProfile] = Field(default=None, description="User profile")
    preferences: UserPreferences = Field(default_factory=UserPreferences, description="User preferences")
    
    # Relationships
    permissions: Set[str] = Field(default_factory=set, description="User permissions")
    tags: Set[str] = Field(default_factory=set, description="User tags")
    
    class Config:
        arbitrary_types_allowed = True
        use_enum_values = True
    
    def __init__(self, **data: Any) -> None:
        super().__init__(**data)
        self._domain_events: List[DomainEvent] = []
    
    @validator('email')
    def validate_email(cls, v: str) -> str:
        """Validate email format."""
        import re
        pattern = r'^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$'
        if not re.match(pattern, v):
            raise ValueError('Invalid email format')
        return v.lower()
    
    @validator('username')
    def validate_username(cls, v: str) -> str:
        """Validate username format."""
        import re
        if not re.match(r'^[a-zA-Z0-9_-]+$', v):
            raise ValueError('Username can only contain letters, numbers, underscores, and hyphens')
        return v.lower()
    
    @root_validator
    def validate_password_reset(cls, values: Dict[str, Any]) -> Dict[str, Any]:
        """Validate password reset token and expiration consistency."""
        token = values.get('password_reset_token')
        expires = values.get('password_reset_expires')
        
        if token and not expires:
            raise ValueError('Password reset token requires expiration date')
        if expires and not token:
            raise ValueError('Password reset expiration requires token')
        
        return values
    
    # Authentication methods
    @classmethod
    def create_user(
        cls,
        username: str,
        email: str,
        password: str,
        role: UserRole = UserRole.USER,
        **kwargs: Any
    ) -> "User":
        """Create a new user with password hashing."""
        user = cls(
            username=username,
            email=email,
            password_hash=cls._hash_password(password),
            role=role,
            email_verification_token=secrets.token_urlsafe(32),
            **kwargs
        )
        
        # Add domain event
        user.add_domain_event(UserCreatedEvent(
            aggregate_id=user.id,
            aggregate_version=user.version,
            username=username,
            email=email,
            role=role
        ))
        
        return user
    
    @staticmethod
    def _hash_password(password: str) -> str:
        """Hash a password using SHA-256 with salt."""
        salt = secrets.token_hex(16)
        password_hash = hashlib.sha256((password + salt).encode()).hexdigest()
        return f"{salt}:{password_hash}"
    
    def verify_password(self, password: str) -> bool:
        """Verify a password against the stored hash."""
        try:
            salt, stored_hash = self.password_hash.split(':')
            password_hash = hashlib.sha256((password + salt).encode()).hexdigest()
            return password_hash == stored_hash
        except ValueError:
            return False
    
    def change_password(self, new_password: str, changed_by: Optional[UUID] = None) -> None:
        """Change the user's password."""
        if not new_password or len(new_password) < 8:
            raise ValidationError("Password must be at least 8 characters long")
        
        old_hash = self.password_hash
        self.password_hash = self._hash_password(new_password)
        self.failed_login_attempts = 0
        self.locked_until = None
        self.password_reset_token = None
        self.password_reset_expires = None
        self.touch()
        
        # Add domain event
        self.add_domain_event(UserPasswordChangedEvent(
            aggregate_id=self.id,
            aggregate_version=self.version,
            changed_by=changed_by or self.id
        ))
    
    def authenticate(self, password: str, ip_address: str) -> bool:
        """Authenticate user with password and handle failed attempts."""
        if self.is_locked:
            raise AuthenticationError("Account is locked due to too many failed attempts")
        
        if self.status == UserStatus.BANNED:
            raise AuthenticationError("Account is banned")
        
        if self.status == UserStatus.SUSPENDED:
            raise AuthenticationError("Account is suspended")
        
        if not self.email_verified:
            raise AuthenticationError("Email address not verified")
        
        if self.verify_password(password):
            # Successful login
            self.failed_login_attempts = 0
            self.locked_until = None
            self.last_login = datetime.utcnow()
            self.touch()
            return True
        else:
            # Failed login
            self.failed_login_attempts += 1
            if self.failed_login_attempts >= 5:
                self.locked_until = datetime.utcnow() + timedelta(minutes=30)
            self.touch()
            return False
    
    # Status and role management
    def change_role(self, new_role: UserRole, changed_by: UUID) -> None:
        """Change the user's role."""
        if new_role == self.role:
            return
        
        old_role = self.role
        self.role = new_role
        self.touch()
        
        # Add domain event
        self.add_domain_event(UserRoleChangedEvent(
            aggregate_id=self.id,
            aggregate_version=self.version,
            old_role=old_role,
            new_role=new_role,
            changed_by=changed_by
        ))
    
    def activate(self) -> None:
        """Activate the user account."""
        if self.status == UserStatus.ACTIVE:
            return
        
        self.status = UserStatus.ACTIVE
        self.touch()
    
    def deactivate(self, reason: Optional[str] = None) -> None:
        """Deactivate the user account."""
        if self.status == UserStatus.INACTIVE:
            return
        
        self.status = UserStatus.INACTIVE
        self.touch()
        
        # Add domain event
        self.add_domain_event(UserDeactivatedEvent(
            aggregate_id=self.id,
            aggregate_version=self.version,
            reason=reason
        ))
    
    def suspend(self, reason: Optional[str] = None) -> None:
        """Suspend the user account."""
        self.status = UserStatus.SUSPENDED
        self.metadata['suspension_reason'] = reason
        self.touch()
    
    def ban(self, reason: Optional[str] = None) -> None:
        """Ban the user account."""
        self.status = UserStatus.BANNED
        self.metadata['ban_reason'] = reason
        self.touch()
    
    # Email verification
    def verify_email(self, token: str) -> bool:
        """Verify email with token."""
        if self.email_verification_token == token:
            self.email_verified = True
            self.email_verification_token = None
            if self.status == UserStatus.PENDING:
                self.status = UserStatus.ACTIVE
            self.touch()
            return True
        return False
    
    def regenerate_verification_token(self) -> str:
        """Generate a new email verification token."""
        self.email_verification_token = secrets.token_urlsafe(32)
        self.touch()
        return self.email_verification_token
    
    # Password reset
    def request_password_reset(self) -> str:
        """Request a password reset and return the token."""
        self.password_reset_token = secrets.token_urlsafe(32)
        self.password_reset_expires = datetime.utcnow() + timedelta(hours=24)
        self.touch()
        return self.password_reset_token
    
    def reset_password_with_token(self, token: str, new_password: str) -> bool:
        """Reset password using a token."""
        if not self.password_reset_token or not self.password_reset_expires:
            return False
        
        if self.password_reset_token != token:
            return False
        
        if datetime.utcnow() > self.password_reset_expires:
            return False
        
        self.change_password(new_password)
        return True
    
    # Permission management
    def add_permission(self, permission: str) -> None:
        """Add a permission to the user."""
        self.permissions.add(permission)
        self.touch()
    
    def remove_permission(self, permission: str) -> None:
        """Remove a permission from the user."""
        self.permissions.discard(permission)
        self.touch()
    
    def has_permission(self, permission: str) -> bool:
        """Check if user has a specific permission."""
        return permission in self.permissions
    
    def has_any_permission(self, permissions: List[str]) -> bool:
        """Check if user has any of the specified permissions."""
        return bool(self.permissions.intersection(permissions))
    
    def has_all_permissions(self, permissions: List[str]) -> bool:
        """Check if user has all of the specified permissions."""
        return set(permissions).issubset(self.permissions)
    
    # Properties
    @property
    def is_active(self) -> bool:
        """Check if user is active."""
        return self.status == UserStatus.ACTIVE
    
    @property
    def is_locked(self) -> bool:
        """Check if user account is locked."""
        return self.locked_until and datetime.utcnow() < self.locked_until
    
    @property
    def is_admin(self) -> bool:
        """Check if user is an admin."""
        return self.role == UserRole.ADMIN
    
    @property
    def is_moderator(self) -> bool:
        """Check if user is a moderator or admin."""
        return self.role in (UserRole.ADMIN, UserRole.MODERATOR)
    
    @property
    def display_name(self) -> str:
        """Get display name (profile full name or username)."""
        if self.profile and self.profile.full_name:
            return self.profile.full_name
        return self.username
    
    # Domain events
    def add_domain_event(self, event: DomainEvent) -> None:
        """Add a domain event."""
        if not hasattr(self, '_domain_events'):
            self._domain_events = []
        self._domain_events.append(event)
    
    def get_domain_events(self) -> List[DomainEvent]:
        """Get domain events generated by this entity."""
        return getattr(self, '_domain_events', []).copy()
    
    def clear_domain_events(self) -> None:
        """Clear all domain events."""
        if hasattr(self, '_domain_events'):
            self._domain_events.clear()
    
    # Business logic validation
    def validate_business_rules(self) -> None:
        """Validate business rules for the user."""
        errors = []
        
        # Admin users must have email verified
        if self.role == UserRole.ADMIN and not self.email_verified:
            errors.append("Admin users must have verified email addresses")
        
        # Active users must have verified email
        if self.status == UserStatus.ACTIVE and not self.email_verified:
            errors.append("Active users must have verified email addresses")
        
        # Banned users cannot have admin role
        if self.status == UserStatus.BANNED and self.role == UserRole.ADMIN:
            errors.append("Banned users cannot have admin role")
        
        if errors:
            raise BusinessLogicError(
                "User business rule violations",
                business_rule="user_validation",
                entity_id=self.id
            )
    
    # Update tracking
    def update_fields(self, **kwargs: Any) -> None:
        """Update user fields and track changes."""
        changes = {}
        for field, value in kwargs.items():
            if hasattr(self, field) and getattr(self, field) != value:
                changes[field] = {
                    'old': getattr(self, field),
                    'new': value
                }
                setattr(self, field, value)
        
        if changes:
            self.touch()
            self.add_domain_event(UserUpdatedEvent(
                aggregate_id=self.id,
                aggregate_version=self.version,
                changes=changes
            ))


# User aggregate builder for complex construction
class UserBuilder:
    """Builder pattern for creating complex User instances."""
    
    def __init__(self) -> None:
        self._username: Optional[str] = None
        self._email: Optional[str] = None
        self._password: Optional[str] = None
        self._role: UserRole = UserRole.USER
        self._profile_data: Dict[str, Any] = {}
        self._preferences_data: Dict[str, Any] = {}
        self._permissions: Set[str] = set()
        self._tags: Set[str] = set()
    
    def with_credentials(self, username: str, email: str, password: str) -> "UserBuilder":
        """Set user credentials."""
        self._username = username
        self._email = email
        self._password = password
        return self
    
    def with_role(self, role: UserRole) -> "UserBuilder":
        """Set user role."""
        self._role = role
        return self
    
    def with_profile(self, **profile_data: Any) -> "UserBuilder":
        """Set profile data."""
        self._profile_data.update(profile_data)
        return self
    
    def with_preferences(self, **preferences_data: Any) -> "UserBuilder":
        """Set preferences data."""
        self._preferences_data.update(preferences_data)
        return self
    
    def with_permission(self, permission: str) -> "UserBuilder":
        """Add a permission."""
        self._permissions.add(permission)
        return self
    
    def with_tag(self, tag: str) -> "UserBuilder":
        """Add a tag."""
        self._tags.add(tag)
        return self
    
    def build(self) -> User:
        """Build the User instance."""
        if not all([self._username, self._email, self._password]):
            raise ValueError("Username, email, and password are required")
        
        user = User.create_user(
            username=self._username,
            email=self._email,
            password=self._password,
            role=self._role
        )
        
        # Set profile if provided
        if self._profile_data:
            user.profile = UserProfile(user_id=user.id, **self._profile_data)
        
        # Set preferences if provided
        if self._preferences_data:
            user.preferences = UserPreferences(**self._preferences_data)
        
        # Add permissions and tags
        user.permissions = self._permissions.copy()
        user.tags = self._tags.copy()
        
        return user 