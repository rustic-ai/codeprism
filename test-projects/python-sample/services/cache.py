"""Advanced caching services with strategy pattern and generics."""

import asyncio
import json
import pickle
import time
from abc import ABC, abstractmethod
from typing import Any, Dict, Generic, List, Optional, Protocol, Type, TypeVar, Union
from datetime import datetime, timedelta
from enum import Enum
from uuid import UUID

import redis
from pydantic import BaseModel, Field

from core import BaseService, ServiceInterface, Result, Option
from core.types import JsonDict, T, K, V
from core.exceptions import CacheError, ConfigurationError, InfrastructureError

# Type variables for cache operations
CacheKey = TypeVar("CacheKey", bound=Union[str, int, UUID])
CacheValue = TypeVar("CacheValue")
SerializableValue = TypeVar("SerializableValue", bound=Union[BaseModel, dict, list, str, int, float, bool])


class CacheStrategy(str, Enum):
    """Cache strategy enumeration."""
    
    LRU = "lru"  # Least Recently Used
    LFU = "lfu"  # Least Frequently Used  
    FIFO = "fifo"  # First In, First Out
    TTL = "ttl"  # Time To Live
    WRITE_THROUGH = "write_through"
    WRITE_BACK = "write_back"
    WRITE_AROUND = "write_around"


class CacheEvent(BaseModel):
    """Cache operation event for analytics."""
    
    operation: str = Field(description="Cache operation type")
    key: str = Field(description="Cache key")
    hit: bool = Field(description="Whether operation was a hit")
    execution_time_ms: float = Field(description="Execution time in milliseconds")
    size_bytes: Optional[int] = Field(default=None, description="Data size in bytes")
    timestamp: datetime = Field(default_factory=datetime.utcnow)


class CacheStats(BaseModel):
    """Cache statistics model."""
    
    hits: int = Field(default=0, description="Number of cache hits")
    misses: int = Field(default=0, description="Number of cache misses")
    total_operations: int = Field(default=0, description="Total operations")
    evictions: int = Field(default=0, description="Number of evictions")
    memory_usage_bytes: int = Field(default=0, description="Memory usage in bytes")
    avg_response_time_ms: float = Field(default=0.0, description="Average response time")
    
    @property
    def hit_rate(self) -> float:
        """Calculate cache hit rate."""
        if self.total_operations == 0:
            return 0.0
        return self.hits / self.total_operations
    
    @property
    def miss_rate(self) -> float:
        """Calculate cache miss rate."""
        return 1.0 - self.hit_rate


class Serializer(Protocol[T]):
    """Protocol for cache value serialization."""
    
    def serialize(self, value: T) -> bytes:
        """Serialize value to bytes."""
        ...
    
    def deserialize(self, data: bytes) -> T:
        """Deserialize bytes to value."""
        ...


class JSONSerializer(Generic[T]):
    """JSON-based serializer for cache values."""
    
    def __init__(self, value_type: Type[T]) -> None:
        self.value_type = value_type
    
    def serialize(self, value: T) -> bytes:
        """Serialize value to JSON bytes."""
        if isinstance(value, BaseModel):
            return value.model_dump_json().encode('utf-8')
        return json.dumps(value, default=str).encode('utf-8')
    
    def deserialize(self, data: bytes) -> T:
        """Deserialize JSON bytes to value."""
        json_str = data.decode('utf-8')
        if issubclass(self.value_type, BaseModel):
            return self.value_type.model_validate_json(json_str)
        return json.loads(json_str)


class PickleSerializer(Generic[T]):
    """Pickle-based serializer for cache values."""
    
    def serialize(self, value: T) -> bytes:
        """Serialize value using pickle."""
        return pickle.dumps(value)
    
    def deserialize(self, data: bytes) -> T:
        """Deserialize pickle bytes to value."""
        return pickle.loads(data)


class CacheEntry(BaseModel, Generic[T]):
    """Cache entry with metadata."""
    
    key: str = Field(description="Cache key")
    value: T = Field(description="Cached value")
    created_at: datetime = Field(default_factory=datetime.utcnow)
    accessed_at: datetime = Field(default_factory=datetime.utcnow)
    access_count: int = Field(default=1, description="Number of times accessed")
    ttl_seconds: Optional[int] = Field(default=None, description="Time to live in seconds")
    size_bytes: int = Field(default=0, description="Entry size in bytes")
    tags: List[str] = Field(default_factory=list, description="Cache entry tags")
    
    class Config:
        arbitrary_types_allowed = True
    
    @property
    def is_expired(self) -> bool:
        """Check if cache entry is expired."""
        if self.ttl_seconds is None:
            return False
        expiry_time = self.created_at + timedelta(seconds=self.ttl_seconds)
        return datetime.utcnow() > expiry_time
    
    def touch(self) -> None:
        """Update access time and count."""
        self.accessed_at = datetime.utcnow()
        self.access_count += 1


class CacheService(ServiceInterface, Generic[K, V], ABC):
    """Abstract base class for cache services."""
    
    def __init__(
        self,
        default_ttl: Optional[int] = None,
        max_size: Optional[int] = None,
        strategy: CacheStrategy = CacheStrategy.LRU,
    ) -> None:
        super().__init__()
        self.default_ttl = default_ttl
        self.max_size = max_size
        self.strategy = strategy
        self.stats = CacheStats()
        self._observers: List[Any] = []  # For observer pattern
    
    @abstractmethod
    async def get(self, key: K) -> Option[V]:
        """Get value from cache."""
        pass
    
    @abstractmethod
    async def set(self, key: K, value: V, ttl: Optional[int] = None) -> bool:
        """Set value in cache."""
        pass
    
    @abstractmethod
    async def delete(self, key: K) -> bool:
        """Delete value from cache."""
        pass
    
    @abstractmethod
    async def exists(self, key: K) -> bool:
        """Check if key exists in cache."""
        pass
    
    @abstractmethod
    async def clear(self) -> None:
        """Clear all cache entries."""
        pass
    
    @abstractmethod
    async def get_size(self) -> int:
        """Get current cache size."""
        pass
    
    # Batch operations
    async def get_many(self, keys: List[K]) -> Dict[K, Option[V]]:
        """Get multiple values from cache."""
        results = {}
        for key in keys:
            results[key] = await self.get(key)
        return results
    
    async def set_many(self, items: Dict[K, V], ttl: Optional[int] = None) -> Dict[K, bool]:
        """Set multiple values in cache."""
        results = {}
        for key, value in items.items():
            results[key] = await self.set(key, value, ttl)
        return results
    
    async def delete_many(self, keys: List[K]) -> Dict[K, bool]:
        """Delete multiple values from cache."""
        results = {}
        for key in keys:
            results[key] = await self.delete(key)
        return results
    
    # Pattern-based operations
    @abstractmethod
    async def get_by_pattern(self, pattern: str) -> Dict[K, V]:
        """Get all entries matching pattern."""
        pass
    
    @abstractmethod
    async def delete_by_pattern(self, pattern: str) -> int:
        """Delete all entries matching pattern."""
        pass
    
    @abstractmethod
    async def get_by_tags(self, tags: List[str]) -> Dict[K, V]:
        """Get all entries with specified tags."""
        pass
    
    # Statistics and monitoring
    def get_stats(self) -> CacheStats:
        """Get cache statistics."""
        return self.stats
    
    def reset_stats(self) -> None:
        """Reset cache statistics."""
        self.stats = CacheStats()
    
    # Observer pattern for cache events
    def add_observer(self, observer: Any) -> None:
        """Add cache event observer."""
        self._observers.append(observer)
    
    def remove_observer(self, observer: Any) -> None:
        """Remove cache event observer."""
        if observer in self._observers:
            self._observers.remove(observer)
    
    def _notify_observers(self, event: CacheEvent) -> None:
        """Notify observers of cache event."""
        for observer in self._observers:
            if hasattr(observer, 'on_cache_event'):
                observer.on_cache_event(event)


class MemoryCacheService(CacheService[str, Any]):
    """In-memory cache service with LRU eviction."""
    
    def __init__(
        self,
        default_ttl: Optional[int] = None,
        max_size: int = 1000,
        strategy: CacheStrategy = CacheStrategy.LRU,
    ) -> None:
        super().__init__(default_ttl, max_size, strategy)
        self._cache: Dict[str, CacheEntry[Any]] = {}
        self._access_order: List[str] = []  # For LRU tracking
    
    async def get(self, key: str) -> Option[Any]:
        """Get value from memory cache."""
        start_time = time.time()
        
        if key not in self._cache:
            self.stats.misses += 1
            self.stats.total_operations += 1
            self._record_event("get", key, False, start_time)
            return Option.none()
        
        entry = self._cache[key]
        
        if entry.is_expired:
            await self.delete(key)
            self.stats.misses += 1
            self.stats.total_operations += 1
            self._record_event("get", key, False, start_time)
            return Option.none()
        
        # Update access information
        entry.touch()
        self._update_access_order(key)
        
        self.stats.hits += 1
        self.stats.total_operations += 1
        self._record_event("get", key, True, start_time)
        
        return Option.some(entry.value)
    
    async def set(self, key: str, value: Any, ttl: Optional[int] = None) -> bool:
        """Set value in memory cache."""
        start_time = time.time()
        
        try:
            # Calculate size (rough estimate)
            size_bytes = len(str(value).encode('utf-8'))
            
            # Check if we need to evict entries
            await self._maybe_evict()
            
            # Create cache entry
            entry = CacheEntry(
                key=key,
                value=value,
                ttl_seconds=ttl or self.default_ttl,
                size_bytes=size_bytes
            )
            
            self._cache[key] = entry
            self._update_access_order(key)
            
            self.stats.memory_usage_bytes += size_bytes
            self._record_event("set", key, True, start_time, size_bytes)
            
            return True
            
        except Exception as e:
            self._record_event("set", key, False, start_time)
            raise CacheError(f"Failed to set cache entry: {str(e)}", cache_key=key, operation="set")
    
    async def delete(self, key: str) -> bool:
        """Delete value from memory cache."""
        if key not in self._cache:
            return False
        
        entry = self._cache[key]
        self.stats.memory_usage_bytes -= entry.size_bytes
        
        del self._cache[key]
        if key in self._access_order:
            self._access_order.remove(key)
        
        return True
    
    async def exists(self, key: str) -> bool:
        """Check if key exists in memory cache."""
        if key not in self._cache:
            return False
        
        entry = self._cache[key]
        if entry.is_expired:
            await self.delete(key)
            return False
        
        return True
    
    async def clear(self) -> None:
        """Clear all cache entries."""
        self._cache.clear()
        self._access_order.clear()
        self.stats.memory_usage_bytes = 0
    
    async def get_size(self) -> int:
        """Get current cache size."""
        return len(self._cache)
    
    async def get_by_pattern(self, pattern: str) -> Dict[str, Any]:
        """Get all entries matching pattern."""
        import re
        regex = re.compile(pattern)
        results = {}
        
        for key, entry in self._cache.items():
            if regex.match(key) and not entry.is_expired:
                entry.touch()
                results[key] = entry.value
        
        return results
    
    async def delete_by_pattern(self, pattern: str) -> int:
        """Delete all entries matching pattern."""
        import re
        regex = re.compile(pattern)
        keys_to_delete = []
        
        for key in self._cache.keys():
            if regex.match(key):
                keys_to_delete.append(key)
        
        for key in keys_to_delete:
            await self.delete(key)
        
        return len(keys_to_delete)
    
    async def get_by_tags(self, tags: List[str]) -> Dict[str, Any]:
        """Get all entries with specified tags."""
        results = {}
        
        for key, entry in self._cache.items():
            if not entry.is_expired and any(tag in entry.tags for tag in tags):
                entry.touch()
                results[key] = entry.value
        
        return results
    
    def _update_access_order(self, key: str) -> None:
        """Update LRU access order."""
        if key in self._access_order:
            self._access_order.remove(key)
        self._access_order.append(key)
    
    async def _maybe_evict(self) -> None:
        """Evict entries if cache is full."""
        if self.max_size and len(self._cache) >= self.max_size:
            if self.strategy == CacheStrategy.LRU:
                await self._evict_lru()
            elif self.strategy == CacheStrategy.LFU:
                await self._evict_lfu()
            elif self.strategy == CacheStrategy.FIFO:
                await self._evict_fifo()
    
    async def _evict_lru(self) -> None:
        """Evict least recently used entry."""
        if self._access_order:
            lru_key = self._access_order[0]
            await self.delete(lru_key)
            self.stats.evictions += 1
    
    async def _evict_lfu(self) -> None:
        """Evict least frequently used entry."""
        if self._cache:
            lfu_key = min(self._cache.keys(), key=lambda k: self._cache[k].access_count)
            await self.delete(lfu_key)
            self.stats.evictions += 1
    
    async def _evict_fifo(self) -> None:
        """Evict first in, first out entry."""
        if self._cache:
            fifo_key = min(self._cache.keys(), key=lambda k: self._cache[k].created_at)
            await self.delete(fifo_key)
            self.stats.evictions += 1
    
    def _record_event(
        self,
        operation: str,
        key: str,
        hit: bool,
        start_time: float,
        size_bytes: Optional[int] = None
    ) -> None:
        """Record cache operation event."""
        execution_time_ms = (time.time() - start_time) * 1000
        
        event = CacheEvent(
            operation=operation,
            key=key,
            hit=hit,
            execution_time_ms=execution_time_ms,
            size_bytes=size_bytes
        )
        
        self._notify_observers(event)


class RedisCacheService(CacheService[str, Any]):
    """Redis-based cache service with advanced features."""
    
    def __init__(
        self,
        redis_url: str = "redis://localhost:6379",
        default_ttl: Optional[int] = None,
        max_size: Optional[int] = None,
        key_prefix: str = "cache:",
        serializer: Optional[Serializer] = None,
    ) -> None:
        super().__init__(default_ttl, max_size)
        self.redis_url = redis_url
        self.key_prefix = key_prefix
        self.serializer = serializer or PickleSerializer()
        self._redis: Optional[redis.Redis] = None
    
    async def initialize(self) -> None:
        """Initialize Redis connection."""
        try:
            self._redis = redis.from_url(self.redis_url, decode_responses=False)
            await self._redis.ping()
            await super().initialize()
        except Exception as e:
            raise ConfigurationError(f"Failed to connect to Redis: {str(e)}")
    
    async def shutdown(self) -> None:
        """Shutdown Redis connection."""
        if self._redis:
            await self._redis.close()
        await super().shutdown()
    
    def _make_key(self, key: str) -> str:
        """Create prefixed Redis key."""
        return f"{self.key_prefix}{key}"
    
    async def get(self, key: str) -> Option[Any]:
        """Get value from Redis cache."""
        if not self._redis:
            raise InfrastructureError("Redis not initialized")
        
        start_time = time.time()
        redis_key = self._make_key(key)
        
        try:
            data = await self._redis.get(redis_key)
            if data is None:
                self.stats.misses += 1
                self.stats.total_operations += 1
                self._record_event("get", key, False, start_time)
                return Option.none()
            
            value = self.serializer.deserialize(data)
            self.stats.hits += 1
            self.stats.total_operations += 1
            self._record_event("get", key, True, start_time)
            
            return Option.some(value)
            
        except Exception as e:
            self._record_event("get", key, False, start_time)
            raise CacheError(f"Failed to get from cache: {str(e)}", cache_key=key, operation="get")
    
    async def set(self, key: str, value: Any, ttl: Optional[int] = None) -> bool:
        """Set value in Redis cache."""
        if not self._redis:
            raise InfrastructureError("Redis not initialized")
        
        start_time = time.time()
        redis_key = self._make_key(key)
        
        try:
            data = self.serializer.serialize(value)
            effective_ttl = ttl or self.default_ttl
            
            if effective_ttl:
                await self._redis.setex(redis_key, effective_ttl, data)
            else:
                await self._redis.set(redis_key, data)
            
            self._record_event("set", key, True, start_time, len(data))
            return True
            
        except Exception as e:
            self._record_event("set", key, False, start_time)
            raise CacheError(f"Failed to set cache entry: {str(e)}", cache_key=key, operation="set")
    
    async def delete(self, key: str) -> bool:
        """Delete value from Redis cache."""
        if not self._redis:
            raise InfrastructureError("Redis not initialized")
        
        redis_key = self._make_key(key)
        result = await self._redis.delete(redis_key)
        return result > 0
    
    async def exists(self, key: str) -> bool:
        """Check if key exists in Redis cache."""
        if not self._redis:
            raise InfrastructureError("Redis not initialized")
        
        redis_key = self._make_key(key)
        return await self._redis.exists(redis_key) > 0
    
    async def clear(self) -> None:
        """Clear all cache entries with prefix."""
        if not self._redis:
            raise InfrastructureError("Redis not initialized")
        
        pattern = f"{self.key_prefix}*"
        keys = await self._redis.keys(pattern)
        if keys:
            await self._redis.delete(*keys)
    
    async def get_size(self) -> int:
        """Get current cache size."""
        if not self._redis:
            raise InfrastructureError("Redis not initialized")
        
        pattern = f"{self.key_prefix}*"
        keys = await self._redis.keys(pattern)
        return len(keys)
    
    async def get_by_pattern(self, pattern: str) -> Dict[str, Any]:
        """Get all entries matching pattern."""
        if not self._redis:
            raise InfrastructureError("Redis not initialized")
        
        redis_pattern = f"{self.key_prefix}{pattern}"
        keys = await self._redis.keys(redis_pattern)
        results = {}
        
        for redis_key in keys:
            data = await self._redis.get(redis_key)
            if data:
                original_key = redis_key[len(self.key_prefix):]
                value = self.serializer.deserialize(data)
                results[original_key] = value
        
        return results
    
    async def delete_by_pattern(self, pattern: str) -> int:
        """Delete all entries matching pattern."""
        if not self._redis:
            raise InfrastructureError("Redis not initialized")
        
        redis_pattern = f"{self.key_prefix}{pattern}"
        keys = await self._redis.keys(redis_pattern)
        
        if keys:
            return await self._redis.delete(*keys)
        return 0
    
    async def get_by_tags(self, tags: List[str]) -> Dict[str, Any]:
        """Get all entries with specified tags (using Redis sets for tag tracking)."""
        if not self._redis:
            raise InfrastructureError("Redis not initialized")
        
        results = {}
        for tag in tags:
            tag_key = f"{self.key_prefix}tag:{tag}"
            tagged_keys = await self._redis.smembers(tag_key)
            
            for redis_key in tagged_keys:
                data = await self._redis.get(redis_key)
                if data:
                    original_key = redis_key[len(self.key_prefix):]
                    value = self.serializer.deserialize(data)
                    results[original_key] = value
        
        return results
    
    def _record_event(
        self,
        operation: str,
        key: str,
        hit: bool,
        start_time: float,
        size_bytes: Optional[int] = None
    ) -> None:
        """Record cache operation event."""
        execution_time_ms = (time.time() - start_time) * 1000
        
        event = CacheEvent(
            operation=operation,
            key=key,
            hit=hit,
            execution_time_ms=execution_time_ms,
            size_bytes=size_bytes
        )
        
        self._notify_observers(event)


# Decorator pattern for cache composition
class CacheDecorator(CacheService[K, V], Generic[K, V]):
    """Base decorator for cache services."""
    
    def __init__(self, cache_service: CacheService[K, V]) -> None:
        super().__init__()
        self._cache_service = cache_service
    
    async def get(self, key: K) -> Option[V]:
        return await self._cache_service.get(key)
    
    async def set(self, key: K, value: V, ttl: Optional[int] = None) -> bool:
        return await self._cache_service.set(key, value, ttl)
    
    async def delete(self, key: K) -> bool:
        return await self._cache_service.delete(key)
    
    async def exists(self, key: K) -> bool:
        return await self._cache_service.exists(key)
    
    async def clear(self) -> None:
        return await self._cache_service.clear()
    
    async def get_size(self) -> int:
        return await self._cache_service.get_size()
    
    async def get_by_pattern(self, pattern: str) -> Dict[K, V]:
        return await self._cache_service.get_by_pattern(pattern)
    
    async def delete_by_pattern(self, pattern: str) -> int:
        return await self._cache_service.delete_by_pattern(pattern)
    
    async def get_by_tags(self, tags: List[str]) -> Dict[K, V]:
        return await self._cache_service.get_by_tags(tags)


class MetricsCollectingCacheDecorator(CacheDecorator[K, V]):
    """Cache decorator that collects detailed metrics."""
    
    def __init__(self, cache_service: CacheService[K, V]) -> None:
        super().__init__(cache_service)
        self._operation_times: List[float] = []
    
    async def get(self, key: K) -> Option[V]:
        start_time = time.time()
        result = await super().get(key)
        self._record_operation_time(time.time() - start_time)
        return result
    
    async def set(self, key: K, value: V, ttl: Optional[int] = None) -> bool:
        start_time = time.time()
        result = await super().set(key, value, ttl)
        self._record_operation_time(time.time() - start_time)
        return result
    
    def _record_operation_time(self, duration: float) -> None:
        """Record operation timing for analytics."""
        self._operation_times.append(duration * 1000)  # Convert to milliseconds
        # Keep only last 1000 operations
        if len(self._operation_times) > 1000:
            self._operation_times = self._operation_times[-1000:]
    
    def get_average_response_time(self) -> float:
        """Get average response time in milliseconds."""
        if not self._operation_times:
            return 0.0
        return sum(self._operation_times) / len(self._operation_times)


# Factory pattern for cache service creation
class CacheServiceFactory:
    """Factory for creating cache service instances."""
    
    @staticmethod
    def create_memory_cache(
        max_size: int = 1000,
        default_ttl: Optional[int] = None,
        strategy: CacheStrategy = CacheStrategy.LRU,
        enable_metrics: bool = True,
    ) -> CacheService:
        """Create memory cache service."""
        cache = MemoryCacheService(default_ttl, max_size, strategy)
        
        if enable_metrics:
            cache = MetricsCollectingCacheDecorator(cache)
        
        return cache
    
    @staticmethod
    def create_redis_cache(
        redis_url: str = "redis://localhost:6379",
        default_ttl: Optional[int] = None,
        key_prefix: str = "cache:",
        enable_metrics: bool = True,
    ) -> CacheService:
        """Create Redis cache service."""
        cache = RedisCacheService(redis_url, default_ttl, key_prefix=key_prefix)
        
        if enable_metrics:
            cache = MetricsCollectingCacheDecorator(cache)
        
        return cache
    
    @staticmethod
    def create_multi_tier_cache(
        l1_max_size: int = 100,
        redis_url: str = "redis://localhost:6379",
        default_ttl: Optional[int] = None,
    ) -> CacheService:
        """Create multi-tier cache (memory + Redis)."""
        # This would implement a multi-tier cache strategy
        # For now, just return a memory cache
        return CacheServiceFactory.create_memory_cache(l1_max_size, default_ttl) 