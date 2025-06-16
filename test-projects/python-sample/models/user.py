"""Enhanced User domain model with advanced patterns and inheritance."""

import hashlib
import secrets
from datetime import datetime, timedelta
from enum import Enum
from typing import Any, Dict, List, Optional, Set
from uuid import UUID, uuid4

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


class UserRole(str, Enum):
    """User role enumeration."""
    
    ADMIN = "admin"
    MODERATOR = "moderator"
    USER = "user"
    GUEST = "guest"


class UserStatus(str, Enum):
    """User status enumeration."""
    
    ACTIVE = "active"
    INACTIVE = "inactive"
    SUSPENDED = "suspended"
    BANNED = "banned"
    PENDING_VERIFICATION = "pending_verification"


class UserPreferences(BaseEntity):
    """User preferences value object."""
    
    theme: str = Field(default="light", description="UI theme preference")
    language: str = Field(default="en", description="Language preference") 
    timezone: str = Field(default="UTC", description="Timezone preference")
    notifications_enabled: bool = Field(default=True, description="Email notifications enabled")
    newsletter_subscribed: bool = Field(default=False, description="Newsletter subscription")
    
    class Config:
        frozen = True  # Immutable value object


class UserProfile(BaseEntity):
    """User profile with extended information."""
    
    user_id: UUID = Field(description="Associated user ID")
    first_name: Optional[str] = Field(default=None, max_length=50, description="First name")
    last_name: Optional[str] = Field(default=None, max_length=50, description="Last name")
    bio: Optional[str] = Field(default=None, max_length=500, description="User biography")
    avatar_url: Optional[str] = Field(default=None, description="Avatar image URL")
    website: Optional[str] = Field(default=None, description="Personal website")
    location: Optional[str] = Field(default=None, max_length=100, description="User location")
    birth_date: Optional[datetime] = Field(default=None, description="Birth date")
    
    @validator('website')
    def validate_website(cls, v: Optional[str]) -> Optional[str]:
        """Validate website URL format."""
        if v and not (v.startswith('http://') or v.startswith('https://')):
            raise ValueError('Website must start with http:// or https://')
        return v
    
    @property
    def full_name(self) -> Optional[str]:
        """Get full name if both first and last names are available."""
        if self.first_name and self.last_name:
            return f"{self.first_name} {self.last_name}"
        return self.first_name or self.last_name
    
    @property
    def age(self) -> Optional[int]:
        """Calculate age from birth date."""
        if self.birth_date:
            today = datetime.utcnow().date()
            return today.year - self.birth_date.year - (
                (today.month, today.day) < (self.birth_date.month, self.birth_date.day)
            )
        return None


class UserSession(BaseEntity):
    """User session for tracking active sessions."""
    
    user_id: UUID = Field(description="Associated user ID")
    session_token: str = Field(description="Session token")
    ip_address: str = Field(description="IP address")
    user_agent: str = Field(description="User agent string")
    expires_at: datetime = Field(description="Session expiration")
    is_active: bool = Field(default=True, description="Session is active")
    
    @classmethod
    def create_session(cls, user_id: UUID, ip_address: str, user_agent: str, duration_hours: int = 24) -> "UserSession":
        """Create a new user session."""
        return cls(
            user_id=user_id,
            session_token=secrets.token_urlsafe(32),
            ip_address=ip_address,
            user_agent=user_agent,
            expires_at=datetime.utcnow() + timedelta(hours=duration_hours)
        )
    
    @property
    def is_expired(self) -> bool:
        """Check if session is expired."""
        return datetime.utcnow() > self.expires_at
    
    def extend_session(self, hours: int = 24) -> None:
        """Extend session expiration."""
        self.expires_at = datetime.utcnow() + timedelta(hours=hours)
        self.touch()
    
    def terminate(self) -> None:
        """Terminate the session."""
        self.is_active = False
        self.touch()


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
    status: UserStatus = Field(default=UserStatus.PENDING_VERIFICATION, description="User status")
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
            if self.status == UserStatus.PENDING_VERIFICATION:
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