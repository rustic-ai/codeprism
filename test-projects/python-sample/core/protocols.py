"""Protocol definitions for core interfaces and capabilities."""

from abc import abstractmethod
from datetime import datetime
from typing import Any, Dict, List, Optional, Protocol, TypeVar
from uuid import UUID

from .types import EntityId, JsonDict

T = TypeVar("T", covariant=True)
K = TypeVar("K")
V = TypeVar("V")


class Identifiable(Protocol[T]):
    """Protocol for entities that have an identity."""

    @property
    def id(self) -> T:
        """Get the entity's unique identifier."""
        ...

    def __eq__(self, other: object) -> bool:
        """Check equality based on identity."""
        ...

    def __hash__(self) -> int:
        """Get hash based on identity."""
        ...


class Timestamped(Protocol):
    """Protocol for entities that track creation and modification times."""

    @property
    def created_at(self) -> datetime:
        """Get the creation timestamp."""
        ...

    @property
    def updated_at(self) -> Optional[datetime]:
        """Get the last update timestamp."""
        ...

    def touch(self) -> None:
        """Update the modification timestamp."""
        ...


class Versioned(Protocol):
    """Protocol for entities that support versioning."""

    @property
    def version(self) -> int:
        """Get the current version number."""
        ...

    def increment_version(self) -> None:
        """Increment the version number."""
        ...


class Serializable(Protocol):
    """Protocol for objects that can be serialized."""

    def to_dict(self) -> JsonDict:
        """Serialize to dictionary."""
        ...

    def to_json(self) -> str:
        """Serialize to JSON string."""
        ...

    @classmethod
    def from_dict(cls, data: JsonDict) -> "Serializable":
        """Deserialize from dictionary."""
        ...

    @classmethod
    def from_json(cls, json_str: str) -> "Serializable":
        """Deserialize from JSON string."""
        ...


class Validatable(Protocol):
    """Protocol for objects that can be validated."""

    def validate(self) -> bool:
        """Validate the object's state."""
        ...

    def get_validation_errors(self) -> List[str]:
        """Get list of validation errors."""
        ...

    @property
    def is_valid(self) -> bool:
        """Check if the object is in a valid state."""
        ...


class Cacheable(Protocol):
    """Protocol for objects that can be cached."""

    def get_cache_key(self) -> str:
        """Get the cache key for this object."""
        ...

    def get_cache_ttl(self) -> Optional[int]:
        """Get the cache time-to-live in seconds."""
        ...

    def invalidate_cache(self) -> None:
        """Invalidate this object's cache."""
        ...


class Auditable(Protocol):
    """Protocol for entities that support audit trails."""

    @property
    def created_by(self) -> Optional[EntityId]:
        """Get the ID of the user who created this entity."""
        ...

    @property
    def updated_by(self) -> Optional[EntityId]:
        """Get the ID of the user who last updated this entity."""
        ...

    def audit_create(self, user_id: EntityId) -> None:
        """Record creation audit information."""
        ...

    def audit_update(self, user_id: EntityId) -> None:
        """Record update audit information."""
        ...


class SoftDeletable(Protocol):
    """Protocol for entities that support soft deletion."""

    @property
    def deleted_at(self) -> Optional[datetime]:
        """Get the deletion timestamp."""
        ...

    @property
    def deleted_by(self) -> Optional[EntityId]:
        """Get the ID of the user who deleted this entity."""
        ...

    @property
    def is_deleted(self) -> bool:
        """Check if the entity is soft deleted."""
        ...

    def soft_delete(self, user_id: Optional[EntityId] = None) -> None:
        """Mark the entity as soft deleted."""
        ...

    def restore(self) -> None:
        """Restore a soft deleted entity."""
        ...


class Traceable(Protocol):
    """Protocol for objects that support distributed tracing."""

    @property
    def trace_id(self) -> Optional[str]:
        """Get the trace ID."""
        ...

    @property
    def span_id(self) -> Optional[str]:
        """Get the span ID."""
        ...

    def set_trace_context(self, trace_id: str, span_id: str) -> None:
        """Set the trace context."""
        ...

    def get_trace_context(self) -> Dict[str, str]:
        """Get the trace context as headers."""
        ...


class Configurable(Protocol):
    """Protocol for components that can be configured."""

    def configure(self, **kwargs: Any) -> None:
        """Configure the component with parameters."""
        ...

    def get_configuration(self) -> JsonDict:
        """Get the current configuration."""
        ...

    @property
    def is_configured(self) -> bool:
        """Check if the component is properly configured."""
        ...


class Initializable(Protocol):
    """Protocol for components that require initialization."""

    async def initialize(self) -> None:
        """Initialize the component."""
        ...

    async def shutdown(self) -> None:
        """Shutdown the component gracefully."""
        ...

    @property
    def is_initialized(self) -> bool:
        """Check if the component is initialized."""
        ...


class HealthCheckable(Protocol):
    """Protocol for components that support health checks."""

    async def health_check(self) -> bool:
        """Perform a health check."""
        ...

    async def ready_check(self) -> bool:
        """Check if the component is ready to serve requests."""
        ...

    def get_health_details(self) -> JsonDict:
        """Get detailed health information."""
        ...


class Retryable(Protocol):
    """Protocol for operations that can be retried."""

    @property
    def max_retries(self) -> int:
        """Get the maximum number of retries."""
        ...

    @property
    def retry_delay(self) -> float:
        """Get the delay between retries in seconds."""
        ...

    def should_retry(self, attempt: int, error: Exception) -> bool:
        """Determine if the operation should be retried."""
        ...


class Lockable(Protocol):
    """Protocol for resources that can be locked."""

    async def acquire_lock(self, timeout: Optional[float] = None) -> bool:
        """Acquire a lock on the resource."""
        ...

    async def release_lock(self) -> None:
        """Release the lock on the resource."""
        ...

    @property
    def is_locked(self) -> bool:
        """Check if the resource is currently locked."""
        ...


class Measurable(Protocol):
    """Protocol for objects that can emit metrics."""

    def increment_counter(self, name: str, value: float = 1.0, tags: Optional[Dict[str, str]] = None) -> None:
        """Increment a counter metric."""
        ...

    def record_gauge(self, name: str, value: float, tags: Optional[Dict[str, str]] = None) -> None:
        """Record a gauge metric."""
        ...

    def record_timing(self, name: str, duration: float, tags: Optional[Dict[str, str]] = None) -> None:
        """Record a timing metric."""
        ...


class Observable(Protocol):
    """Protocol for objects that can be observed."""

    def subscribe(self, observer: "Observer") -> None:
        """Subscribe an observer."""
        ...

    def unsubscribe(self, observer: "Observer") -> None:
        """Unsubscribe an observer."""
        ...

    def notify_observers(self, event: Any) -> None:
        """Notify all observers of an event."""
        ...


class Comparable(Protocol):
    """Protocol for objects that can be compared."""

    def __lt__(self, other: "Comparable") -> bool:
        """Less than comparison."""
        ...

    def __le__(self, other: "Comparable") -> bool:
        """Less than or equal comparison."""
        ...

    def __gt__(self, other: "Comparable") -> bool:
        """Greater than comparison."""
        ...

    def __ge__(self, other: "Comparable") -> bool:
        """Greater than or equal comparison."""
        ...


class Copyable(Protocol[T]):
    """Protocol for objects that can be copied."""

    def copy(self) -> T:
        """Create a shallow copy."""
        ...

    def deep_copy(self) -> T:
        """Create a deep copy."""
        ...


class Filterable(Protocol[T]):
    """Protocol for collections that can be filtered."""

    def filter(self, predicate: Any) -> List[T]:
        """Filter items using a predicate."""
        ...

    def find(self, predicate: Any) -> Optional[T]:
        """Find the first item matching a predicate."""
        ...

    def find_all(self, predicate: Any) -> List[T]:
        """Find all items matching a predicate."""
        ...


class Transformable(Protocol[T, V]):
    """Protocol for objects that can be transformed."""

    def transform(self, transformer: Any) -> V:
        """Transform the object using a transformer."""
        ...

    def map(self, mapper: Any) -> V:
        """Map the object to another type."""
        ...


class Persistable(Protocol):
    """Protocol for objects that can be persisted."""

    async def save(self) -> None:
        """Save the object to persistent storage."""
        ...

    async def delete(self) -> None:
        """Delete the object from persistent storage."""
        ...

    async def reload(self) -> None:
        """Reload the object from persistent storage."""
        ...

    @property
    def is_persisted(self) -> bool:
        """Check if the object is persisted."""
        ...

    @property
    def is_dirty(self) -> bool:
        """Check if the object has unsaved changes."""
        ...


class Indexable(Protocol[K, V]):
    """Protocol for objects that support indexing."""

    def __getitem__(self, key: K) -> V:
        """Get item by key."""
        ...

    def __setitem__(self, key: K, value: V) -> None:
        """Set item by key."""
        ...

    def __delitem__(self, key: K) -> None:
        """Delete item by key."""
        ...

    def __contains__(self, key: K) -> bool:
        """Check if key exists."""
        ...


class Iterable(Protocol[T]):
    """Protocol for objects that can be iterated."""

    def __iter__(self) -> "Iterator[T]":
        """Get an iterator."""
        ...

    def __next__(self) -> T:
        """Get the next item."""
        ...


class Sized(Protocol):
    """Protocol for objects that have a size."""

    def __len__(self) -> int:
        """Get the size."""
        ...

    @property
    def size(self) -> int:
        """Get the size as a property."""
        ...

    @property
    def is_empty(self) -> bool:
        """Check if the object is empty."""
        ...


# Combined protocols for common use cases
class Entity(Identifiable[UUID], Timestamped, Versioned, Auditable, SoftDeletable, Protocol):
    """Protocol combining common entity capabilities."""

    pass


class ValueObject(Serializable, Validatable, Comparable, Copyable, Protocol):
    """Protocol for value objects."""

    pass


class Repository(Initializable, HealthCheckable, Measurable, Protocol[T]):
    """Protocol for repository pattern implementations."""

    async def find_by_id(self, id: EntityId) -> Optional[T]:
        """Find an entity by its ID."""
        ...

    async def find_all(self) -> List[T]:
        """Find all entities."""
        ...

    async def save(self, entity: T) -> T:
        """Save an entity."""
        ...

    async def delete(self, entity: T) -> None:
        """Delete an entity."""
        ...


class Service(Initializable, HealthCheckable, Configurable, Measurable, Protocol):
    """Protocol for service layer implementations."""

    pass


class UseCase(Measurable, Traceable, Protocol[T]):
    """Protocol for use case implementations."""

    async def execute(self, *args: Any, **kwargs: Any) -> T:
        """Execute the use case."""
        ...


# from typing imports
Iterator = Any  # This would normally be typing.Iterator 