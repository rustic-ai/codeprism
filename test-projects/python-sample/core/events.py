"""Event system for domain-driven design and event sourcing."""

import asyncio
import uuid
from abc import ABC, abstractmethod
from datetime import datetime
from typing import Any, Callable, Dict, List, Optional, Type, TypeVar, Union
from uuid import UUID

from pydantic import BaseModel, Field

from .types import JsonDict

EventType = TypeVar("EventType", bound="BaseEvent")
HandlerType = TypeVar("HandlerType", bound="EventHandler")


class BaseEvent(BaseModel, ABC):
    """Base class for all events in the system."""

    event_id: UUID = Field(default_factory=uuid.uuid4, description="Unique event identifier")
    event_type: str = Field(description="Type of the event")
    occurred_at: datetime = Field(default_factory=datetime.utcnow, description="When the event occurred")
    correlation_id: Optional[UUID] = Field(default=None, description="Correlation ID for tracking related events")
    causation_id: Optional[UUID] = Field(default=None, description="ID of the event that caused this event")
    metadata: JsonDict = Field(default_factory=dict, description="Additional metadata")

    class Config:
        arbitrary_types_allowed = True
        use_enum_values = True

    def __init__(self, **data: Any) -> None:
        if "event_type" not in data:
            data["event_type"] = self.__class__.__name__
        super().__init__(**data)

    def with_correlation_id(self, correlation_id: UUID) -> "BaseEvent":
        """Create a copy of this event with a correlation ID."""
        return self.model_copy(update={"correlation_id": correlation_id})

    def with_causation_id(self, causation_id: UUID) -> "BaseEvent":
        """Create a copy of this event with a causation ID."""
        return self.model_copy(update={"causation_id": causation_id})

    def to_dict(self) -> JsonDict:
        """Convert event to dictionary."""
        return self.model_dump(mode="json")

    def to_json(self) -> str:
        """Convert event to JSON string."""
        return self.model_dump_json()

    @classmethod
    def from_dict(cls, data: JsonDict) -> "BaseEvent":
        """Create event from dictionary."""
        return cls.model_validate(data)

    @classmethod
    def from_json(cls, json_str: str) -> "BaseEvent":
        """Create event from JSON string."""
        return cls.model_validate_json(json_str)


class DomainEvent(BaseEvent):
    """Base class for domain events."""

    aggregate_id: UUID = Field(description="ID of the aggregate that generated this event")
    aggregate_type: str = Field(description="Type of the aggregate")
    aggregate_version: int = Field(description="Version of the aggregate when event was generated")

    def __init__(self, **data: Any) -> None:
        if "aggregate_type" not in data and hasattr(self, "_aggregate_type"):
            data["aggregate_type"] = self._aggregate_type
        super().__init__(**data)


class IntegrationEvent(BaseEvent):
    """Base class for integration events (events sent between bounded contexts)."""

    source_context: str = Field(description="Source bounded context")
    destination_context: Optional[str] = Field(default=None, description="Destination bounded context")
    published_at: datetime = Field(default_factory=datetime.utcnow, description="When the event was published")

    def __init__(self, **data: Any) -> None:
        super().__init__(**data)


class EventHandler(ABC):
    """Abstract base class for event handlers."""

    @abstractmethod
    async def handle(self, event: BaseEvent) -> None:
        """Handle an event."""
        pass

    @abstractmethod
    def can_handle(self, event_type: Type[BaseEvent]) -> bool:
        """Check if this handler can handle the given event type."""
        pass


class AsyncEventHandler(EventHandler):
    """Base class for asynchronous event handlers."""

    def __init__(self, event_type: Type[BaseEvent]) -> None:
        self.event_type = event_type

    def can_handle(self, event_type: Type[BaseEvent]) -> bool:
        """Check if this handler can handle the given event type."""
        return issubclass(event_type, self.event_type)

    @abstractmethod
    async def handle(self, event: BaseEvent) -> None:
        """Handle an event asynchronously."""
        pass


class FunctionalEventHandler(AsyncEventHandler):
    """Event handler that wraps a function."""

    def __init__(
        self,
        event_type: Type[BaseEvent],
        handler_func: Callable[[BaseEvent], None],
    ) -> None:
        super().__init__(event_type)
        self.handler_func = handler_func

    async def handle(self, event: BaseEvent) -> None:
        """Handle an event using the wrapped function."""
        if asyncio.iscoroutinefunction(self.handler_func):
            await self.handler_func(event)
        else:
            self.handler_func(event)


class EventSubscriber:
    """Manages event subscriptions for a specific subscriber."""

    def __init__(self, subscriber_id: str) -> None:
        self.subscriber_id = subscriber_id
        self.handlers: Dict[Type[BaseEvent], List[EventHandler]] = {}
        self.is_active = True

    def subscribe(self, event_type: Type[BaseEvent], handler: EventHandler) -> None:
        """Subscribe to an event type with a handler."""
        if event_type not in self.handlers:
            self.handlers[event_type] = []
        self.handlers[event_type].append(handler)

    def unsubscribe(self, event_type: Type[BaseEvent], handler: EventHandler) -> None:
        """Unsubscribe a handler from an event type."""
        if event_type in self.handlers:
            self.handlers[event_type] = [h for h in self.handlers[event_type] if h != handler]
            if not self.handlers[event_type]:
                del self.handlers[event_type]

    def get_handlers(self, event_type: Type[BaseEvent]) -> List[EventHandler]:
        """Get all handlers for an event type."""
        handlers = []
        for registered_type, type_handlers in self.handlers.items():
            if issubclass(event_type, registered_type):
                handlers.extend(type_handlers)
        return handlers

    def activate(self) -> None:
        """Activate this subscriber."""
        self.is_active = True

    def deactivate(self) -> None:
        """Deactivate this subscriber."""
        self.is_active = False


class EventPublisher(ABC):
    """Abstract base class for event publishers."""

    @abstractmethod
    async def publish(self, event: BaseEvent) -> None:
        """Publish an event."""
        pass

    @abstractmethod
    async def publish_batch(self, events: List[BaseEvent]) -> None:
        """Publish multiple events as a batch."""
        pass


class InMemoryEventPublisher(EventPublisher):
    """In-memory event publisher for testing and simple scenarios."""

    def __init__(self) -> None:
        self.published_events: List[BaseEvent] = []

    async def publish(self, event: BaseEvent) -> None:
        """Publish a single event."""
        self.published_events.append(event)

    async def publish_batch(self, events: List[BaseEvent]) -> None:
        """Publish multiple events."""
        self.published_events.extend(events)

    def get_published_events(self) -> List[BaseEvent]:
        """Get all published events."""
        return self.published_events.copy()

    def clear(self) -> None:
        """Clear all published events."""
        self.published_events.clear()


class EventBus:
    """Central event bus for managing event publishing and subscription."""

    def __init__(self, publisher: Optional[EventPublisher] = None) -> None:
        self.publisher = publisher or InMemoryEventPublisher()
        self.subscribers: Dict[str, EventSubscriber] = {}
        self.middleware: List[Callable[[BaseEvent], BaseEvent]] = []
        self.global_handlers: List[EventHandler] = []

    def add_subscriber(self, subscriber: EventSubscriber) -> None:
        """Add a subscriber to the event bus."""
        self.subscribers[subscriber.subscriber_id] = subscriber

    def remove_subscriber(self, subscriber_id: str) -> None:
        """Remove a subscriber from the event bus."""
        self.subscribers.pop(subscriber_id, None)

    def subscribe(
        self,
        subscriber_id: str,
        event_type: Type[BaseEvent],
        handler: EventHandler,
    ) -> None:
        """Subscribe to an event type."""
        if subscriber_id not in self.subscribers:
            self.subscribers[subscriber_id] = EventSubscriber(subscriber_id)
        self.subscribers[subscriber_id].subscribe(event_type, handler)

    def subscribe_function(
        self,
        subscriber_id: str,
        event_type: Type[BaseEvent],
        handler_func: Callable[[BaseEvent], None],
    ) -> None:
        """Subscribe a function to an event type."""
        handler = FunctionalEventHandler(event_type, handler_func)
        self.subscribe(subscriber_id, event_type, handler)

    def unsubscribe(
        self,
        subscriber_id: str,
        event_type: Type[BaseEvent],
        handler: EventHandler,
    ) -> None:
        """Unsubscribe from an event type."""
        if subscriber_id in self.subscribers:
            self.subscribers[subscriber_id].unsubscribe(event_type, handler)

    def add_middleware(self, middleware: Callable[[BaseEvent], BaseEvent]) -> None:
        """Add middleware to process events before publishing."""
        self.middleware.append(middleware)

    def add_global_handler(self, handler: EventHandler) -> None:
        """Add a global handler that receives all events."""
        self.global_handlers.append(handler)

    async def publish(self, event: BaseEvent) -> None:
        """Publish an event to all subscribers."""
        # Apply middleware
        processed_event = event
        for middleware in self.middleware:
            processed_event = middleware(processed_event)

        # Publish to external publisher
        await self.publisher.publish(processed_event)

        # Notify subscribers
        await self._notify_subscribers(processed_event)

    async def publish_batch(self, events: List[BaseEvent]) -> None:
        """Publish multiple events."""
        processed_events = []
        for event in events:
            processed_event = event
            for middleware in self.middleware:
                processed_event = middleware(processed_event)
            processed_events.append(processed_event)

        # Publish to external publisher
        await self.publisher.publish_batch(processed_events)

        # Notify subscribers
        for event in processed_events:
            await self._notify_subscribers(event)

    async def _notify_subscribers(self, event: BaseEvent) -> None:
        """Notify all relevant subscribers of an event."""
        event_type = type(event)
        handlers = []

        # Collect handlers from subscribers
        for subscriber in self.subscribers.values():
            if subscriber.is_active:
                handlers.extend(subscriber.get_handlers(event_type))

        # Add global handlers
        handlers.extend(self.global_handlers)

        # Execute handlers concurrently
        tasks = []
        for handler in handlers:
            if handler.can_handle(event_type):
                tasks.append(handler.handle(event))

        if tasks:
            await asyncio.gather(*tasks, return_exceptions=True)


class EventStore(ABC):
    """Abstract base class for event stores."""

    @abstractmethod
    async def append(self, aggregate_id: UUID, events: List[DomainEvent], expected_version: int) -> None:
        """Append events to the store."""
        pass

    @abstractmethod
    async def get_events(self, aggregate_id: UUID, from_version: int = 0) -> List[DomainEvent]:
        """Get events for an aggregate."""
        pass

    @abstractmethod
    async def get_all_events(self, from_version: int = 0) -> List[DomainEvent]:
        """Get all events from the store."""
        pass


class InMemoryEventStore(EventStore):
    """In-memory event store for testing and simple scenarios."""

    def __init__(self) -> None:
        self.events: Dict[UUID, List[DomainEvent]] = {}
        self.global_version = 0

    async def append(self, aggregate_id: UUID, events: List[DomainEvent], expected_version: int) -> None:
        """Append events to the store."""
        if aggregate_id not in self.events:
            self.events[aggregate_id] = []

        current_version = len(self.events[aggregate_id])
        if current_version != expected_version:
            raise Exception(f"Concurrency conflict: expected version {expected_version}, got {current_version}")

        # Assign global versions to events
        for event in events:
            self.global_version += 1
            event.metadata["global_version"] = self.global_version

        self.events[aggregate_id].extend(events)

    async def get_events(self, aggregate_id: UUID, from_version: int = 0) -> List[DomainEvent]:
        """Get events for an aggregate."""
        if aggregate_id not in self.events:
            return []
        return self.events[aggregate_id][from_version:]

    async def get_all_events(self, from_version: int = 0) -> List[DomainEvent]:
        """Get all events from the store."""
        all_events = []
        for events in self.events.values():
            all_events.extend(events)
        
        # Sort by global version
        all_events.sort(key=lambda e: e.metadata.get("global_version", 0))
        return all_events[from_version:]


class EventSourcingRepository(ABC):
    """Base class for event-sourced repositories."""

    def __init__(self, event_store: EventStore, event_bus: EventBus) -> None:
        self.event_store = event_store
        self.event_bus = event_bus

    @abstractmethod
    def create_aggregate(self, aggregate_id: UUID) -> Any:
        """Create a new aggregate instance."""
        pass

    async def load(self, aggregate_id: UUID) -> Any:
        """Load an aggregate from its events."""
        events = await self.event_store.get_events(aggregate_id)
        if not events:
            raise Exception(f"Aggregate {aggregate_id} not found")

        aggregate = self.create_aggregate(aggregate_id)
        for event in events:
            aggregate.apply(event)
        aggregate.mark_events_as_committed()
        return aggregate

    async def save(self, aggregate: Any) -> None:
        """Save an aggregate's uncommitted events."""
        uncommitted_events = aggregate.get_uncommitted_events()
        if not uncommitted_events:
            return

        expected_version = aggregate.version - len(uncommitted_events)
        await self.event_store.append(aggregate.id, uncommitted_events, expected_version)

        # Publish events
        for event in uncommitted_events:
            await self.event_bus.publish(event)

        aggregate.mark_events_as_committed()


# Event middleware functions
def correlation_middleware(event: BaseEvent) -> BaseEvent:
    """Middleware to ensure events have correlation IDs."""
    if event.correlation_id is None:
        event.correlation_id = uuid.uuid4()
    return event


def logging_middleware(event: BaseEvent) -> BaseEvent:
    """Middleware to log events."""
    print(f"Event published: {event.event_type} at {event.occurred_at}")
    return event


def metrics_middleware(event: BaseEvent) -> BaseEvent:
    """Middleware to record metrics for events."""
    # This would integrate with a metrics system
    event.metadata["metrics_recorded"] = True
    return event 