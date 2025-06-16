"""Base classes and abstract interfaces for the core infrastructure."""

import copy
import uuid
from abc import ABC, abstractmethod
from datetime import datetime
from typing import Any, ClassVar, Dict, Generic, List, Optional, Type, TypeVar
from uuid import UUID

from pydantic import BaseModel, Field, computed_field

from .protocols import (
    Auditable,
    Cacheable,
    Identifiable,
    Measurable,
    Repository,
    Service,
    SoftDeletable,
    Timestamped,
    UseCase,
    Validatable,
    Versioned,
)
from .types import EntityId, JsonDict, Result

T = TypeVar("T")
E = TypeVar("E", bound="BaseEntity")
R = TypeVar("R", bound="Repository")
S = TypeVar("S", bound="Service")
U = TypeVar("U", bound="UseCase")


class BaseEntity(
    BaseModel,
    Identifiable[UUID],
    Timestamped,
    Versioned,
    Auditable,
    SoftDeletable,
    Validatable,
    Cacheable,
    ABC,
):
    """Base class for all domain entities with comprehensive capabilities."""

    id: UUID = Field(default_factory=uuid.uuid4, description="Unique identifier")
    created_at: datetime = Field(default_factory=datetime.utcnow, description="Creation timestamp")
    updated_at: Optional[datetime] = Field(default=None, description="Last update timestamp")
    version: int = Field(default=1, description="Version number for optimistic locking")
    
    # Audit fields
    created_by: Optional[EntityId] = Field(default=None, description="ID of the user who created this entity")
    updated_by: Optional[EntityId] = Field(default=None, description="ID of the user who last updated this entity")
    
    # Soft deletion fields
    deleted_at: Optional[datetime] = Field(default=None, description="Deletion timestamp")
    deleted_by: Optional[EntityId] = Field(default=None, description="ID of the user who deleted this entity")
    
    # Metadata
    metadata: JsonDict = Field(default_factory=dict, description="Additional metadata")

    class Config:
        arbitrary_types_allowed = True
        use_enum_values = True
        validate_assignment = True
        extra = "forbid"

    def __eq__(self, other: object) -> bool:
        """Check equality based on identity."""
        if not isinstance(other, BaseEntity):
            return False
        return self.id == other.id

    def __hash__(self) -> int:
        """Get hash based on identity."""
        return hash(self.id)

    def __str__(self) -> str:
        """String representation."""
        return f"{self.__class__.__name__}(id={self.id})"

    def __repr__(self) -> str:
        """Detailed string representation."""
        return f"{self.__class__.__name__}(id={self.id}, version={self.version})"

    # Timestamped protocol implementation
    def touch(self) -> None:
        """Update the modification timestamp."""
        self.updated_at = datetime.utcnow()
        self.increment_version()

    # Versioned protocol implementation
    def increment_version(self) -> None:
        """Increment the version number."""
        self.version += 1

    # Auditable protocol implementation
    def audit_create(self, user_id: EntityId) -> None:
        """Record creation audit information."""
        self.created_by = user_id

    def audit_update(self, user_id: EntityId) -> None:
        """Record update audit information."""
        self.updated_by = user_id
        self.touch()

    # SoftDeletable protocol implementation
    @property
    def is_deleted(self) -> bool:
        """Check if the entity is soft deleted."""
        return self.deleted_at is not None

    def soft_delete(self, user_id: Optional[EntityId] = None) -> None:
        """Mark the entity as soft deleted."""
        self.deleted_at = datetime.utcnow()
        self.deleted_by = user_id
        self.touch()

    def restore(self) -> None:
        """Restore a soft deleted entity."""
        self.deleted_at = None
        self.deleted_by = None
        self.touch()

    # Validatable protocol implementation
    def validate(self) -> bool:
        """Validate the entity's state."""
        errors = self.get_validation_errors()
        return len(errors) == 0

    def get_validation_errors(self) -> List[str]:
        """Get list of validation errors."""
        errors = []
        
        if self.created_at > datetime.utcnow():
            errors.append("Creation date cannot be in the future")
        
        if self.updated_at and self.updated_at < self.created_at:
            errors.append("Update date cannot be before creation date")
        
        if self.version < 1:
            errors.append("Version must be positive")
        
        if self.deleted_at and self.deleted_at < self.created_at:
            errors.append("Deletion date cannot be before creation date")
        
        return errors

    @property
    def is_valid(self) -> bool:
        """Check if the entity is in a valid state."""
        return self.validate()

    # Cacheable protocol implementation
    def get_cache_key(self) -> str:
        """Get the cache key for this entity."""
        return f"{self.__class__.__name__}:{self.id}"

    def get_cache_ttl(self) -> Optional[int]:
        """Get the cache time-to-live in seconds."""
        return 3600  # 1 hour default

    def invalidate_cache(self) -> None:
        """Invalidate this entity's cache."""
        # Implementation would depend on cache backend
        pass

    # Serialization methods
    def to_dict(self) -> JsonDict:
        """Serialize to dictionary."""
        return self.model_dump(mode="json")

    def to_json(self) -> str:
        """Serialize to JSON string."""
        return self.model_dump_json()

    @classmethod
    def from_dict(cls, data: JsonDict) -> "BaseEntity":
        """Deserialize from dictionary."""
        return cls.model_validate(data)

    @classmethod
    def from_json(cls, json_str: str) -> "BaseEntity":
        """Deserialize from JSON string."""
        return cls.model_validate_json(json_str)

    # Domain-specific methods (to be overridden by subclasses)
    @abstractmethod
    def get_domain_events(self) -> List["DomainEvent"]:
        """Get domain events generated by this entity."""
        pass

    @abstractmethod
    def clear_domain_events(self) -> None:
        """Clear all domain events."""
        pass


class AggregateRoot(BaseEntity):
    """Base class for aggregate roots in Domain-Driven Design."""

    _domain_events: List["DomainEvent"] = []

    def __init__(self, **data: Any) -> None:
        super().__init__(**data)
        self._domain_events = []

    def add_domain_event(self, event: "DomainEvent") -> None:
        """Add a domain event."""
        self._domain_events.append(event)

    def get_domain_events(self) -> List["DomainEvent"]:
        """Get domain events generated by this entity."""
        return self._domain_events.copy()

    def clear_domain_events(self) -> None:
        """Clear all domain events."""
        self._domain_events.clear()


class ValueObject(BaseModel, ABC):
    """Base class for value objects."""

    class Config:
        frozen = True  # Value objects are immutable
        arbitrary_types_allowed = True
        use_enum_values = True

    def __eq__(self, other: object) -> bool:
        """Value objects are equal if all their attributes are equal."""
        if not isinstance(other, self.__class__):
            return False
        return self.__dict__ == other.__dict__

    def __hash__(self) -> int:
        """Hash based on all attributes."""
        return hash(tuple(sorted(self.__dict__.items())))

    def copy(self) -> "ValueObject":
        """Create a shallow copy."""
        return copy.copy(self)

    def deep_copy(self) -> "ValueObject":
        """Create a deep copy."""
        return copy.deepcopy(self)


class RepositoryInterface(Repository[E], ABC, Generic[E]):
    """Abstract base class for repository pattern implementations."""

    def __init__(self) -> None:
        self._initialized = False
        self._health_status = True

    @abstractmethod
    async def find_by_id(self, id: EntityId) -> Optional[E]:
        """Find an entity by its ID."""
        pass

    @abstractmethod
    async def find_all(self) -> List[E]:
        """Find all entities."""
        pass

    @abstractmethod
    async def save(self, entity: E) -> E:
        """Save an entity."""
        pass

    @abstractmethod
    async def delete(self, entity: E) -> None:
        """Delete an entity."""
        pass

    @abstractmethod
    async def find_by_criteria(self, criteria: Dict[str, Any]) -> List[E]:
        """Find entities by custom criteria."""
        pass

    @abstractmethod
    async def count(self) -> int:
        """Count total number of entities."""
        pass

    @abstractmethod
    async def exists(self, id: EntityId) -> bool:
        """Check if entity exists."""
        pass

    # Initializable protocol implementation
    async def initialize(self) -> None:
        """Initialize the repository."""
        self._initialized = True

    async def shutdown(self) -> None:
        """Shutdown the repository gracefully."""
        self._initialized = False

    @property
    def is_initialized(self) -> bool:
        """Check if the repository is initialized."""
        return self._initialized

    # HealthCheckable protocol implementation
    async def health_check(self) -> bool:
        """Perform a health check."""
        return self._health_status

    async def ready_check(self) -> bool:
        """Check if the repository is ready to serve requests."""
        return self.is_initialized and self._health_status

    def get_health_details(self) -> JsonDict:
        """Get detailed health information."""
        return {
            "initialized": self.is_initialized,
            "healthy": self._health_status,
            "ready": self.is_initialized and self._health_status,
        }

    # Measurable protocol implementation
    def increment_counter(self, name: str, value: float = 1.0, tags: Optional[Dict[str, str]] = None) -> None:
        """Increment a counter metric."""
        # Implementation would depend on metrics backend
        pass

    def record_gauge(self, name: str, value: float, tags: Optional[Dict[str, str]] = None) -> None:
        """Record a gauge metric."""
        # Implementation would depend on metrics backend
        pass

    def record_timing(self, name: str, duration: float, tags: Optional[Dict[str, str]] = None) -> None:
        """Record a timing metric."""
        # Implementation would depend on metrics backend
        pass


class BaseRepository(RepositoryInterface[E], Generic[E]):
    """Base implementation for repository pattern."""

    def __init__(self, entity_type: Type[E]) -> None:
        super().__init__()
        self.entity_type = entity_type
        self._storage: Dict[EntityId, E] = {}

    async def find_by_id(self, id: EntityId) -> Optional[E]:
        """Find an entity by its ID."""
        self.increment_counter("repository.find_by_id")
        return self._storage.get(id)

    async def find_all(self) -> List[E]:
        """Find all entities."""
        self.increment_counter("repository.find_all")
        return list(self._storage.values())

    async def save(self, entity: E) -> E:
        """Save an entity."""
        self.increment_counter("repository.save")
        self._storage[entity.id] = entity  # type: ignore
        return entity

    async def delete(self, entity: E) -> None:
        """Delete an entity."""
        self.increment_counter("repository.delete")
        self._storage.pop(entity.id, None)  # type: ignore

    async def find_by_criteria(self, criteria: Dict[str, Any]) -> List[E]:
        """Find entities by custom criteria."""
        self.increment_counter("repository.find_by_criteria")
        # Simple implementation - would be more sophisticated in real repository
        return [entity for entity in self._storage.values() 
                if all(getattr(entity, key, None) == value for key, value in criteria.items())]

    async def count(self) -> int:
        """Count total number of entities."""
        self.increment_counter("repository.count")
        return len(self._storage)

    async def exists(self, id: EntityId) -> bool:
        """Check if entity exists."""
        self.increment_counter("repository.exists")
        return id in self._storage


class ServiceInterface(Service, ABC):
    """Abstract base class for service layer implementations."""

    def __init__(self) -> None:
        self._initialized = False
        self._configured = False
        self._config: JsonDict = {}

    # Initializable protocol implementation
    async def initialize(self) -> None:
        """Initialize the service."""
        self._initialized = True

    async def shutdown(self) -> None:
        """Shutdown the service gracefully."""
        self._initialized = False

    @property
    def is_initialized(self) -> bool:
        """Check if the service is initialized."""
        return self._initialized

    # HealthCheckable protocol implementation
    async def health_check(self) -> bool:
        """Perform a health check."""
        return self.is_initialized

    async def ready_check(self) -> bool:
        """Check if the service is ready to serve requests."""
        return self.is_initialized and self.is_configured

    def get_health_details(self) -> JsonDict:
        """Get detailed health information."""
        return {
            "initialized": self.is_initialized,
            "configured": self.is_configured,
            "ready": self.is_initialized and self.is_configured,
        }

    # Configurable protocol implementation
    def configure(self, **kwargs: Any) -> None:
        """Configure the service with parameters."""
        self._config.update(kwargs)
        self._configured = True

    def get_configuration(self) -> JsonDict:
        """Get the current configuration."""
        return self._config.copy()

    @property
    def is_configured(self) -> bool:
        """Check if the service is properly configured."""
        return self._configured

    # Measurable protocol implementation
    def increment_counter(self, name: str, value: float = 1.0, tags: Optional[Dict[str, str]] = None) -> None:
        """Increment a counter metric."""
        # Implementation would depend on metrics backend
        pass

    def record_gauge(self, name: str, value: float, tags: Optional[Dict[str, str]] = None) -> None:
        """Record a gauge metric."""
        # Implementation would depend on metrics backend
        pass

    def record_timing(self, name: str, duration: float, tags: Optional[Dict[str, str]] = None) -> None:
        """Record a timing metric."""
        # Implementation would depend on metrics backend
        pass


class BaseService(ServiceInterface):
    """Base implementation for service layer."""

    def __init__(self, name: str) -> None:
        super().__init__()
        self.name = name


class UseCaseInterface(UseCase[T], ABC, Generic[T]):
    """Abstract base class for use case implementations."""

    def __init__(self) -> None:
        self._trace_id: Optional[str] = None
        self._span_id: Optional[str] = None

    @abstractmethod
    async def execute(self, *args: Any, **kwargs: Any) -> T:
        """Execute the use case."""
        pass

    # Traceable protocol implementation
    @property
    def trace_id(self) -> Optional[str]:
        """Get the trace ID."""
        return self._trace_id

    @property
    def span_id(self) -> Optional[str]:
        """Get the span ID."""
        return self._span_id

    def set_trace_context(self, trace_id: str, span_id: str) -> None:
        """Set the trace context."""
        self._trace_id = trace_id
        self._span_id = span_id

    def get_trace_context(self) -> Dict[str, str]:
        """Get the trace context as headers."""
        context = {}
        if self._trace_id:
            context["X-Trace-Id"] = self._trace_id
        if self._span_id:
            context["X-Span-Id"] = self._span_id
        return context

    # Measurable protocol implementation
    def increment_counter(self, name: str, value: float = 1.0, tags: Optional[Dict[str, str]] = None) -> None:
        """Increment a counter metric."""
        # Implementation would depend on metrics backend
        pass

    def record_gauge(self, name: str, value: float, tags: Optional[Dict[str, str]] = None) -> None:
        """Record a gauge metric."""
        # Implementation would depend on metrics backend
        pass

    def record_timing(self, name: str, duration: float, tags: Optional[Dict[str, str]] = None) -> None:
        """Record a timing metric."""
        # Implementation would depend on metrics backend
        pass


class BaseUseCase(UseCaseInterface[T], Generic[T]):
    """Base implementation for use cases."""

    def __init__(self, name: str) -> None:
        super().__init__()
        self.name = name

    async def execute(self, *args: Any, **kwargs: Any) -> T:
        """Execute the use case with metrics and tracing."""
        self.increment_counter(f"usecase.{self.name}.executed")
        
        start_time = datetime.utcnow()
        try:
            result = await self._execute_impl(*args, **kwargs)
            self.increment_counter(f"usecase.{self.name}.success")
            return result
        except Exception as e:
            self.increment_counter(f"usecase.{self.name}.error")
            raise
        finally:
            duration = (datetime.utcnow() - start_time).total_seconds()
            self.record_timing(f"usecase.{self.name}.duration", duration)

    @abstractmethod
    async def _execute_impl(self, *args: Any, **kwargs: Any) -> T:
        """Internal implementation of the use case logic."""
        pass


class EntityManager(Generic[E]):
    """Manager for handling entity lifecycle and repository operations."""

    def __init__(self, repository: RepositoryInterface[E]) -> None:
        self.repository = repository
        self._unit_of_work: List[E] = []

    async def get_by_id(self, id: EntityId) -> Optional[E]:
        """Get entity by ID."""
        return await self.repository.find_by_id(id)

    async def save(self, entity: E) -> E:
        """Save entity."""
        saved_entity = await self.repository.save(entity)
        self._unit_of_work.append(saved_entity)
        return saved_entity

    async def delete(self, entity: E) -> None:
        """Delete entity."""
        await self.repository.delete(entity)

    async def commit(self) -> None:
        """Commit all pending changes."""
        # In a real implementation, this would handle transactions
        self._unit_of_work.clear()

    async def rollback(self) -> None:
        """Rollback all pending changes."""
        # In a real implementation, this would handle transaction rollback
        self._unit_of_work.clear()

    def get_pending_changes(self) -> List[E]:
        """Get entities with pending changes."""
        return self._unit_of_work.copy()


# Forward declaration for domain events
class DomainEvent(BaseModel, ABC):
    """Base class for domain events."""

    event_id: UUID = Field(default_factory=uuid.uuid4)
    occurred_at: datetime = Field(default_factory=datetime.utcnow)
    aggregate_id: EntityId
    aggregate_type: str
    event_version: int = Field(default=1)

    class Config:
        arbitrary_types_allowed = True 