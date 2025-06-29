#!/usr/bin/env python3
"""
Test file for unused code detection
This file contains intentional unused code to test the find_unused_code MCP tool
"""

# Unused imports
import os
import sys
import json
import pickle
import threading
import multiprocessing
from datetime import datetime, timedelta
from typing import List, Dict, Optional, Union, Tuple
from dataclasses import dataclass
from abc import ABC, abstractmethod
import requests
import sqlite3
import hashlib
import base64
import uuid
import re
import math
import random
import time

# Used imports
from pathlib import Path

# Unused constants
UNUSED_API_URL = "https://api.unused.com"
UNUSED_MAX_RETRIES = 5
UNUSED_TIMEOUT = 30
UNUSED_CACHE_SIZE = 1000
UNUSED_DEBUG_MODE = True

# Used constants
USED_VERSION = "1.0.0"

# Unused classes
class UnusedBaseClass(ABC):
    """Unused abstract base class."""
    
    def __init__(self):
        self.created_at = datetime.now()
        self.id = str(uuid.uuid4())
    
    @abstractmethod
    def process(self):
        """Abstract method that's never implemented."""
        pass
    
    def get_id(self):
        """Unused method."""
        return self.id

class UnusedDataProcessor(UnusedBaseClass):
    """Unused data processor class."""
    
    def __init__(self, data_source: str):
        super().__init__()
        self.data_source = data_source
        self.processed_count = 0
    
    def process(self):
        """Implementation of abstract method - but never called."""
        self.processed_count += 1
        return f"Processed data from {self.data_source}"
    
    def get_stats(self):
        """Unused method."""
        return {
            "processed_count": self.processed_count,
            "data_source": self.data_source
        }

class UnusedLogger:
    """Unused logging class."""
    
    def __init__(self, log_file: str):
        self.log_file = log_file
        self.entries = []
    
    def log(self, level: str, message: str):
        """Unused logging method."""
        entry = {
            "timestamp": datetime.now().isoformat(),
            "level": level,
            "message": message
        }
        self.entries.append(entry)
    
    def save_to_file(self):
        """Unused save method."""
        with open(self.log_file, 'w') as f:
            json.dump(self.entries, f, indent=2)

# Unused dataclass
@dataclass
class UnusedUserProfile:
    """Unused user profile dataclass."""
    user_id: int
    username: str
    email: str
    age: int
    preferences: Dict[str, str]
    created_at: datetime = None
    
    def __post_init__(self):
        if self.created_at is None:
            self.created_at = datetime.now()
    
    def to_dict(self):
        """Unused conversion method."""
        return {
            "user_id": self.user_id,
            "username": self.username,
            "email": self.email,
            "age": self.age,
            "preferences": self.preferences,
            "created_at": self.created_at.isoformat()
        }

# Unused functions
def unused_calculate_hash(data: str) -> str:
    """Unused hash calculation function."""
    return hashlib.sha256(data.encode()).hexdigest()

def unused_encode_data(data: dict) -> str:
    """Unused data encoding function."""
    json_str = json.dumps(data)
    encoded = base64.b64encode(json_str.encode()).decode()
    return encoded

def unused_decode_data(encoded_data: str) -> dict:
    """Unused data decoding function."""
    decoded_bytes = base64.b64decode(encoded_data.encode())
    json_str = decoded_bytes.decode()
    return json.loads(json_str)

def unused_validate_email(email: str) -> bool:
    """Unused email validation function."""
    pattern = r'^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$'
    return bool(re.match(pattern, email))

def unused_generate_password(length: int = 12) -> str:
    """Unused password generation function."""
    characters = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*"
    return ''.join(random.choice(characters) for _ in range(length))

def unused_calculate_distance(point1: Tuple[float, float], point2: Tuple[float, float]) -> float:
    """Unused distance calculation function."""
    x1, y1 = point1
    x2, y2 = point2
    return math.sqrt((x2 - x1)**2 + (y2 - y1)**2)

def unused_format_timestamp(timestamp: datetime) -> str:
    """Unused timestamp formatting function."""
    return timestamp.strftime("%Y-%m-%d %H:%M:%S")

def unused_parse_config_file(file_path: str) -> dict:
    """Unused config parsing function."""
    with open(file_path, 'r') as f:
        return json.load(f)

def unused_save_to_pickle(data: any, file_path: str) -> None:
    """Unused pickle save function."""
    with open(file_path, 'wb') as f:
        pickle.dump(data, f)

def unused_load_from_pickle(file_path: str) -> any:
    """Unused pickle load function."""
    with open(file_path, 'rb') as f:
        return pickle.load(f)

def unused_retry_operation(operation, max_retries: int = 3, delay: float = 1.0):
    """Unused retry decorator function."""
    def wrapper(*args, **kwargs):
        for attempt in range(max_retries):
            try:
                return operation(*args, **kwargs)
            except Exception as e:
                if attempt == max_retries - 1:
                    raise e
                time.sleep(delay * (2 ** attempt))
    return wrapper

def unused_fetch_api_data(url: str, headers: dict = None) -> dict:
    """Unused API fetch function."""
    if headers is None:
        headers = {}
    
    response = requests.get(url, headers=headers, timeout=30)
    response.raise_for_status()
    return response.json()

def unused_database_connection() -> sqlite3.Connection:
    """Unused database connection function."""
    conn = sqlite3.connect(':memory:')
    return conn

def unused_create_thread(target_function, args: tuple = ()):
    """Unused thread creation function."""
    thread = threading.Thread(target=target_function, args=args)
    thread.daemon = True
    return thread

def unused_parallel_process(function, items: list, num_processes: int = 4):
    """Unused parallel processing function."""
    with multiprocessing.Pool(num_processes) as pool:
        results = pool.map(function, items)
    return results

# Unused variables
unused_global_counter = 0
unused_cache = {}
unused_config = {
    "debug": True,
    "log_level": "INFO",
    "max_connections": 100
}
unused_user_sessions = []
unused_temp_data = []

# Partially used function (has unused parameters)
def partially_unused_function(used_param: str, unused_param1: int, unused_param2: dict) -> str:
    """Function with unused parameters."""
    return f"Processing: {used_param}"

# Function with unused local variables
def function_with_unused_locals() -> str:
    """Function with unused local variables."""
    used_var = "This is used"
    unused_var1 = "This is not used"
    unused_var2 = 42
    unused_var3 = {"key": "value"}
    unused_var4 = [1, 2, 3, 4, 5]
    
    return used_var

# Unused nested function
def unused_outer_function():
    """Unused outer function with nested function."""
    
    def unused_inner_function(x: int) -> int:
        """Unused inner function."""
        return x * 2
    
    def another_unused_inner(y: str) -> str:
        """Another unused inner function."""
        return y.upper()
    
    # These nested functions are defined but never called
    return "Outer function result"

# Used function (to demonstrate contrast)
def used_function() -> str:
    """This function is actually used."""
    return f"Version: {USED_VERSION}"

# Unused lambda functions
unused_lambda1 = lambda x: x * 2
unused_lambda2 = lambda a, b: a + b
unused_lambda3 = lambda text: text.strip().lower()

# Unused list comprehensions stored in variables
unused_squares = [x**2 for x in range(10)]
unused_evens = [x for x in range(20) if x % 2 == 0]
unused_words = [word.upper() for word in ["hello", "world", "python"]]

# Unused exception classes
class UnusedException(Exception):
    """Unused custom exception."""
    pass

class UnusedValidationError(UnusedException):
    """Unused validation error."""
    pass

class UnusedConnectionError(UnusedException):
    """Unused connection error."""
    pass

# Unused generator function
def unused_fibonacci_generator(n: int):
    """Unused Fibonacci generator."""
    a, b = 0, 1
    for _ in range(n):
        yield a
        a, b = b, a + b

# Unused decorator
def unused_timing_decorator(func):
    """Unused timing decorator."""
    def wrapper(*args, **kwargs):
        start_time = time.time()
        result = func(*args, **kwargs)
        end_time = time.time()
        print(f"{func.__name__} took {end_time - start_time:.4f} seconds")
        return result
    return wrapper

# Unused context manager
class UnusedFileManager:
    """Unused context manager for file operations."""
    
    def __init__(self, filename: str, mode: str = 'r'):
        self.filename = filename
        self.mode = mode
        self.file = None
    
    def __enter__(self):
        self.file = open(self.filename, self.mode)
        return self.file
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        if self.file:
            self.file.close()

# Main function that only uses a few items
def main():
    """Main function that only uses select items."""
    # Only use a few items to demonstrate unused code detection
    print(used_function())
    print(f"Working with version: {USED_VERSION}")
    
    # Call function with unused parameters
    result = partially_unused_function("test data", 123, {"unused": "dict"})
    print(result)
    
    # Call function with unused locals
    message = function_with_unused_locals()
    print(message)
    
    # Use Path import
    current_path = Path(__file__)
    print(f"Current file: {current_path.name}")

if __name__ == "__main__":
    main() 