"""Advanced type definitions and generics for the core infrastructure."""

from typing import (
    Any,
    Awaitable,
    Callable,
    Dict,
    Generic,
    List,
    Optional,
    Protocol,
    Type,
    TypeVar,
    Union,
)
from uuid import UUID

from pydantic import BaseModel

# Basic type aliases
JsonDict = Dict[str, Any]
EntityId = Union[str, int, UUID]
ID = TypeVar("ID", bound=Union[str, int, UUID])

# Generic type variables
T = TypeVar("T")
K = TypeVar("K")
V = TypeVar("V")
E = TypeVar("E", bound=BaseModel)  # Entity type bound to Pydantic BaseModel
R = TypeVar("R")  # Result type
P = TypeVar("P")  # Payload type

# Repository and Service type variables
EntityType = TypeVar("EntityType", bound="BaseEntity")
RepositoryType = TypeVar("RepositoryType", bound="RepositoryInterface")
ServiceType = TypeVar("ServiceType", bound="ServiceInterface")
UseCaseType = TypeVar("UseCaseType", bound="UseCaseInterface")

# Event and Message types
EventType = TypeVar("EventType", bound="BaseEvent")
MessageType = TypeVar("MessageType", bound=BaseModel)

# Handler types
MessageHandler = Callable[[MessageType], Awaitable[Optional[R]]]
EventHandler = Callable[[EventType], Awaitable[None]]
AsyncHandler = Callable[..., Awaitable[Any]]
SyncHandler = Callable[..., Any]

# Registry types
RegistryType = TypeVar("RegistryType")
ServiceRegistry = Dict[Type[T], T]

# Result and Option types
class Result(Generic[T, E]):
    """A generic Result type for error handling without exceptions."""

    def __init__(self, value: Optional[T] = None, error: Optional[E] = None) -> None:
        if value is not None and error is not None:
            raise ValueError("Result cannot have both value and error")
        if value is None and error is None:
            raise ValueError("Result must have either value or error")
        self._value = value
        self._error = error

    @property
    def is_ok(self) -> bool:
        """Check if the result contains a value."""
        return self._value is not None

    @property
    def is_err(self) -> bool:
        """Check if the result contains an error."""
        return self._error is not None

    @property
    def value(self) -> T:
        """Get the value, raising an exception if error."""
        if self._error is not None:
            raise RuntimeError(f"Result contains error: {self._error}")
        return self._value  # type: ignore

    @property
    def error(self) -> E:
        """Get the error, raising an exception if value."""
        if self._value is not None:
            raise RuntimeError("Result contains value, not error")
        return self._error  # type: ignore

    def unwrap_or(self, default: T) -> T:
        """Get the value or return a default."""
        return self._value if self._value is not None else default

    def map(self, func: Callable[[T], R]) -> "Result[R, E]":
        """Transform the value if present."""
        if self.is_ok:
            try:
                return Result.ok(func(self.value))
            except Exception as e:
                return Result.err(e)  # type: ignore
        return Result.err(self.error)

    def and_then(self, func: Callable[[T], "Result[R, E]"]) -> "Result[R, E]":
        """Chain operations that return Results."""
        if self.is_ok:
            return func(self.value)
        return Result.err(self.error)

    @classmethod
    def ok(cls, value: T) -> "Result[T, E]":
        """Create a successful Result."""
        return cls(value=value)

    @classmethod
    def err(cls, error: E) -> "Result[T, E]":
        """Create an error Result."""
        return cls(error=error)


class Option(Generic[T]):
    """A generic Option type for handling nullable values."""

    def __init__(self, value: Optional[T] = None) -> None:
        self._value = value

    @property
    def is_some(self) -> bool:
        """Check if the option contains a value."""
        return self._value is not None

    @property
    def is_none(self) -> bool:
        """Check if the option is empty."""
        return self._value is None

    @property
    def value(self) -> T:
        """Get the value, raising an exception if None."""
        if self._value is None:
            raise RuntimeError("Option is None")
        return self._value

    def unwrap_or(self, default: T) -> T:
        """Get the value or return a default."""
        return self._value if self._value is not None else default

    def map(self, func: Callable[[T], R]) -> "Option[R]":
        """Transform the value if present."""
        if self.is_some:
            return Option.some(func(self.value))
        return Option.none()

    def and_then(self, func: Callable[[T], "Option[R]"]) -> "Option[R]":
        """Chain operations that return Options."""
        if self.is_some:
            return func(self.value)
        return Option.none()

    @classmethod
    def some(cls, value: T) -> "Option[T]":
        """Create an Option with a value."""
        return cls(value=value)

    @classmethod
    def none(cls) -> "Option[T]":
        """Create an empty Option."""
        return cls()


# Utility type for pagination
class PageInfo(BaseModel):
    """Pagination information."""

    total: int
    page: int
    page_size: int
    has_next: bool
    has_previous: bool

    @property
    def total_pages(self) -> int:
        """Calculate total number of pages."""
        return (self.total + self.page_size - 1) // self.page_size


class PaginatedResult(BaseModel, Generic[T]):
    """Generic paginated result container."""

    items: List[T]
    page_info: PageInfo

    class Config:
        arbitrary_types_allowed = True


# Type for dependency injection
DependencyProvider = Callable[[], T]
DependencyFactory = Callable[..., T]

# Builder pattern type
BuilderType = TypeVar("BuilderType", bound="Builder")


class Builder(Generic[T]):
    """Generic builder pattern base class."""

    def build(self) -> T:
        """Build the final object."""
        raise NotImplementedError("Builder must implement build method")


# Strategy pattern types
StrategyType = TypeVar("StrategyType", bound="Strategy")


class Strategy(Protocol):
    """Protocol for strategy pattern."""

    def execute(self, *args: Any, **kwargs: Any) -> Any:
        """Execute the strategy."""
        ...


# Observer pattern types
ObserverType = TypeVar("ObserverType", bound="Observer")
SubjectType = TypeVar("SubjectType", bound="Subject")


class Observer(Protocol):
    """Protocol for observer pattern."""

    def update(self, subject: "Subject", *args: Any, **kwargs: Any) -> None:
        """Handle subject updates."""
        ...


class Subject(Protocol):
    """Protocol for subject in observer pattern."""

    def attach(self, observer: Observer) -> None:
        """Attach an observer."""
        ...

    def detach(self, observer: Observer) -> None:
        """Detach an observer."""
        ...

    def notify(self, *args: Any, **kwargs: Any) -> None:
        """Notify all observers."""
        ...


# Command pattern types
class Command(Protocol[T]):
    """Protocol for command pattern."""

    def execute(self) -> T:
        """Execute the command."""
        ...

    def undo(self) -> None:
        """Undo the command."""
        ...


# State machine types
StateType = TypeVar("StateType")
TransitionType = TypeVar("TransitionType")

# Type aliases for common patterns
ResultType = Result[T, Exception]
AsyncResultType = Awaitable[Result[T, Exception]]
OptionType = Option[T]
AsyncOptionType = Awaitable[Option[T]] 