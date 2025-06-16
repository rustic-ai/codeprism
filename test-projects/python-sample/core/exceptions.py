"""Exception hierarchy for the core infrastructure."""

from typing import Any, Dict, List, Optional
from uuid import UUID


class ApplicationError(Exception):
    """Base exception for all application errors."""

    def __init__(
        self,
        message: str,
        error_code: Optional[str] = None,
        details: Optional[Dict[str, Any]] = None,
        cause: Optional[Exception] = None,
    ) -> None:
        super().__init__(message)
        self.message = message
        self.error_code = error_code or self.__class__.__name__
        self.details = details or {}
        self.cause = cause

    def to_dict(self) -> Dict[str, Any]:
        """Convert exception to dictionary representation."""
        return {
            "error_type": self.__class__.__name__,
            "error_code": self.error_code,
            "message": self.message,
            "details": self.details,
            "cause": str(self.cause) if self.cause else None,
        }

    def __str__(self) -> str:
        """String representation of the exception."""
        parts = [self.message]
        if self.error_code:
            parts.append(f"Code: {self.error_code}")
        if self.details:
            parts.append(f"Details: {self.details}")
        return " | ".join(parts)


class DomainError(ApplicationError):
    """Base exception for domain-related errors."""

    pass


class BusinessLogicError(DomainError):
    """Exception for business logic violations."""

    def __init__(
        self,
        message: str,
        business_rule: Optional[str] = None,
        entity_id: Optional[UUID] = None,
        **kwargs: Any,
    ) -> None:
        super().__init__(message, **kwargs)
        self.business_rule = business_rule
        self.entity_id = entity_id
        if business_rule:
            self.details["business_rule"] = business_rule
        if entity_id:
            self.details["entity_id"] = str(entity_id)


class ValidationError(DomainError):
    """Exception for validation failures."""

    def __init__(
        self,
        message: str,
        field_errors: Optional[Dict[str, List[str]]] = None,
        **kwargs: Any,
    ) -> None:
        super().__init__(message, **kwargs)
        self.field_errors = field_errors or {}
        if self.field_errors:
            self.details["field_errors"] = self.field_errors

    def add_field_error(self, field: str, error: str) -> None:
        """Add a field-specific validation error."""
        if field not in self.field_errors:
            self.field_errors[field] = []
        self.field_errors[field].append(error)
        self.details["field_errors"] = self.field_errors


class NotFoundError(DomainError):
    """Exception for when a requested entity is not found."""

    def __init__(
        self,
        entity_type: str,
        entity_id: Any,
        message: Optional[str] = None,
        **kwargs: Any,
    ) -> None:
        self.entity_type = entity_type
        self.entity_id = entity_id
        default_message = f"{entity_type} with ID '{entity_id}' not found"
        super().__init__(message or default_message, **kwargs)
        self.details["entity_type"] = entity_type
        self.details["entity_id"] = str(entity_id)


class ConflictError(DomainError):
    """Exception for when an operation conflicts with current state."""

    def __init__(
        self,
        message: str,
        conflicting_entity_id: Optional[Any] = None,
        current_version: Optional[int] = None,
        expected_version: Optional[int] = None,
        **kwargs: Any,
    ) -> None:
        super().__init__(message, **kwargs)
        self.conflicting_entity_id = conflicting_entity_id
        self.current_version = current_version
        self.expected_version = expected_version
        
        if conflicting_entity_id:
            self.details["conflicting_entity_id"] = str(conflicting_entity_id)
        if current_version is not None:
            self.details["current_version"] = current_version
        if expected_version is not None:
            self.details["expected_version"] = expected_version


class ConcurrencyError(ConflictError):
    """Exception for optimistic locking failures."""

    def __init__(
        self,
        entity_type: str,
        entity_id: Any,
        current_version: int,
        expected_version: int,
        **kwargs: Any,
    ) -> None:
        message = (
            f"Concurrency conflict for {entity_type} '{entity_id}': "
            f"expected version {expected_version}, but current is {current_version}"
        )
        super().__init__(
            message,
            conflicting_entity_id=entity_id,
            current_version=current_version,
            expected_version=expected_version,
            **kwargs,
        )
        self.entity_type = entity_type
        self.details["entity_type"] = entity_type


class AuthenticationError(ApplicationError):
    """Exception for authentication failures."""

    def __init__(
        self,
        message: str = "Authentication failed",
        auth_method: Optional[str] = None,
        **kwargs: Any,
    ) -> None:
        super().__init__(message, **kwargs)
        self.auth_method = auth_method
        if auth_method:
            self.details["auth_method"] = auth_method


class AuthorizationError(ApplicationError):
    """Exception for authorization failures."""

    def __init__(
        self,
        message: str = "Access denied",
        required_permission: Optional[str] = None,
        resource_id: Optional[Any] = None,
        user_id: Optional[Any] = None,
        **kwargs: Any,
    ) -> None:
        super().__init__(message, **kwargs)
        self.required_permission = required_permission
        self.resource_id = resource_id
        self.user_id = user_id
        
        if required_permission:
            self.details["required_permission"] = required_permission
        if resource_id:
            self.details["resource_id"] = str(resource_id)
        if user_id:
            self.details["user_id"] = str(user_id)


class InfrastructureError(ApplicationError):
    """Base exception for infrastructure-related errors."""

    pass


class DatabaseError(InfrastructureError):
    """Exception for database-related errors."""

    def __init__(
        self,
        message: str,
        operation: Optional[str] = None,
        table_name: Optional[str] = None,
        **kwargs: Any,
    ) -> None:
        super().__init__(message, **kwargs)
        self.operation = operation
        self.table_name = table_name
        
        if operation:
            self.details["operation"] = operation
        if table_name:
            self.details["table_name"] = table_name


class ExternalServiceError(InfrastructureError):
    """Exception for external service communication errors."""

    def __init__(
        self,
        service_name: str,
        message: str,
        status_code: Optional[int] = None,
        response_body: Optional[str] = None,
        **kwargs: Any,
    ) -> None:
        super().__init__(message, **kwargs)
        self.service_name = service_name
        self.status_code = status_code
        self.response_body = response_body
        
        self.details["service_name"] = service_name
        if status_code:
            self.details["status_code"] = status_code
        if response_body:
            self.details["response_body"] = response_body


class CacheError(InfrastructureError):
    """Exception for cache-related errors."""

    def __init__(
        self,
        message: str,
        cache_key: Optional[str] = None,
        operation: Optional[str] = None,
        **kwargs: Any,
    ) -> None:
        super().__init__(message, **kwargs)
        self.cache_key = cache_key
        self.operation = operation
        
        if cache_key:
            self.details["cache_key"] = cache_key
        if operation:
            self.details["operation"] = operation


class ConfigurationError(ApplicationError):
    """Exception for configuration-related errors."""

    def __init__(
        self,
        message: str,
        config_key: Optional[str] = None,
        config_value: Optional[Any] = None,
        **kwargs: Any,
    ) -> None:
        super().__init__(message, **kwargs)
        self.config_key = config_key
        self.config_value = config_value
        
        if config_key:
            self.details["config_key"] = config_key
        if config_value is not None:
            self.details["config_value"] = str(config_value)


class RateLimitError(ApplicationError):
    """Exception for rate limiting violations."""

    def __init__(
        self,
        message: str = "Rate limit exceeded",
        limit: Optional[int] = None,
        window: Optional[int] = None,
        retry_after: Optional[int] = None,
        **kwargs: Any,
    ) -> None:
        super().__init__(message, **kwargs)
        self.limit = limit
        self.window = window
        self.retry_after = retry_after
        
        if limit:
            self.details["limit"] = limit
        if window:
            self.details["window"] = window
        if retry_after:
            self.details["retry_after"] = retry_after


class ResourceExhaustedError(ApplicationError):
    """Exception for when resources are exhausted."""

    def __init__(
        self,
        resource_type: str,
        message: Optional[str] = None,
        current_usage: Optional[float] = None,
        max_capacity: Optional[float] = None,
        **kwargs: Any,
    ) -> None:
        self.resource_type = resource_type
        default_message = f"{resource_type} resource exhausted"
        super().__init__(message or default_message, **kwargs)
        self.current_usage = current_usage
        self.max_capacity = max_capacity
        
        self.details["resource_type"] = resource_type
        if current_usage is not None:
            self.details["current_usage"] = current_usage
        if max_capacity is not None:
            self.details["max_capacity"] = max_capacity


class TimeoutError(ApplicationError):
    """Exception for operation timeouts."""

    def __init__(
        self,
        operation: str,
        timeout_seconds: float,
        message: Optional[str] = None,
        **kwargs: Any,
    ) -> None:
        self.operation = operation
        self.timeout_seconds = timeout_seconds
        default_message = f"Operation '{operation}' timed out after {timeout_seconds} seconds"
        super().__init__(message or default_message, **kwargs)
        
        self.details["operation"] = operation
        self.details["timeout_seconds"] = timeout_seconds


class CircuitBreakerError(ApplicationError):
    """Exception for circuit breaker activation."""

    def __init__(
        self,
        service_name: str,
        failure_count: int,
        threshold: int,
        message: Optional[str] = None,
        **kwargs: Any,
    ) -> None:
        self.service_name = service_name
        self.failure_count = failure_count
        self.threshold = threshold
        default_message = (
            f"Circuit breaker open for service '{service_name}': "
            f"{failure_count} failures exceeded threshold of {threshold}"
        )
        super().__init__(message or default_message, **kwargs)
        
        self.details["service_name"] = service_name
        self.details["failure_count"] = failure_count
        self.details["threshold"] = threshold


# Exception hierarchy for async operations
class AsyncOperationError(ApplicationError):
    """Base exception for asynchronous operation errors."""

    def __init__(
        self,
        message: str,
        operation_id: Optional[str] = None,
        **kwargs: Any,
    ) -> None:
        super().__init__(message, **kwargs)
        self.operation_id = operation_id
        if operation_id:
            self.details["operation_id"] = operation_id


class TaskExecutionError(AsyncOperationError):
    """Exception for task execution failures."""

    def __init__(
        self,
        task_name: str,
        message: str,
        retry_count: int = 0,
        max_retries: int = 0,
        **kwargs: Any,
    ) -> None:
        super().__init__(message, **kwargs)
        self.task_name = task_name
        self.retry_count = retry_count
        self.max_retries = max_retries
        
        self.details["task_name"] = task_name
        self.details["retry_count"] = retry_count
        self.details["max_retries"] = max_retries


# Helper functions for exception handling
def create_validation_error(field_errors: Dict[str, List[str]]) -> ValidationError:
    """Create a validation error from field errors."""
    message = "Validation failed"
    if field_errors:
        error_count = sum(len(errors) for errors in field_errors.values())
        message = f"Validation failed with {error_count} error(s)"
    
    return ValidationError(message, field_errors=field_errors)


def create_not_found_error(entity_type: str, entity_id: Any) -> NotFoundError:
    """Create a not found error for an entity."""
    return NotFoundError(entity_type, entity_id)


def create_concurrency_error(
    entity_type: str,
    entity_id: Any,
    current_version: int,
    expected_version: int,
) -> ConcurrencyError:
    """Create a concurrency error for optimistic locking failure."""
    return ConcurrencyError(entity_type, entity_id, current_version, expected_version) 