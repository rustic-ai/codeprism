#!/usr/bin/env python3
"""
Test file for duplicate code detection
This file contains intentional code duplicates to test the find_duplicates MCP tool
"""

import json
import requests
from datetime import datetime
from typing import List, Dict, Optional

# Duplicate Pattern 1: User validation
def validate_user_input(username, email, password):
    """Validate user input for registration."""
    errors = []
    
    if not username or len(username) < 3:
        errors.append("Username must be at least 3 characters long")
    
    if not email or "@" not in email:
        errors.append("Invalid email format")
    
    if not password or len(password) < 8:
        errors.append("Password must be at least 8 characters long")
    
    return errors

def validate_admin_input(username, email, password):
    """Validate admin input for registration."""
    errors = []
    
    if not username or len(username) < 3:
        errors.append("Username must be at least 3 characters long")
    
    if not email or "@" not in email:
        errors.append("Invalid email format")
    
    if not password or len(password) < 8:
        errors.append("Password must be at least 8 characters long")
    
    return errors

def validate_moderator_input(username, email, password):
    """Validate moderator input for registration."""
    errors = []
    
    if not username or len(username) < 3:
        errors.append("Username must be at least 3 characters long")
    
    if not email or "@" not in email:
        errors.append("Invalid email format")
    
    if not password or len(password) < 8:
        errors.append("Password must be at least 8 characters long")
    
    return errors

# Duplicate Pattern 2: Database connection logic
class UserDatabase:
    def __init__(self):
        self.connection = None
    
    def connect(self):
        """Connect to the database."""
        try:
            # Simulate database connection
            self.connection = "user_db_connection"
            print("Connected to user database")
            return True
        except Exception as e:
            print(f"Failed to connect to user database: {e}")
            return False
    
    def disconnect(self):
        """Disconnect from the database."""
        if self.connection:
            self.connection = None
            print("Disconnected from user database")

class ProductDatabase:
    def __init__(self):
        self.connection = None
    
    def connect(self):
        """Connect to the database."""
        try:
            # Simulate database connection
            self.connection = "product_db_connection"
            print("Connected to product database")
            return True
        except Exception as e:
            print(f"Failed to connect to product database: {e}")
            return False
    
    def disconnect(self):
        """Disconnect from the database."""
        if self.connection:
            self.connection = None
            print("Disconnected from product database")

class OrderDatabase:
    def __init__(self):
        self.connection = None
    
    def connect(self):
        """Connect to the database."""
        try:
            # Simulate database connection
            self.connection = "order_db_connection"
            print("Connected to order database")
            return True
        except Exception as e:
            print(f"Failed to connect to order database: {e}")
            return False
    
    def disconnect(self):
        """Disconnect from the database."""
        if self.connection:
            self.connection = None
            print("Disconnected from order database")

# Duplicate Pattern 3: API request handling
def fetch_user_data(user_id: int) -> Dict:
    """Fetch user data from API."""
    try:
        url = f"https://api.example.com/users/{user_id}"
        headers = {
            "Authorization": "Bearer token",
            "Content-Type": "application/json"
        }
        
        response = requests.get(url, headers=headers, timeout=30)
        
        if response.status_code == 200:
            return response.json()
        elif response.status_code == 404:
            return {"error": "User not found"}
        else:
            return {"error": f"API request failed with status {response.status_code}"}
            
    except requests.RequestException as e:
        return {"error": f"Request failed: {str(e)}"}
    except json.JSONDecodeError:
        return {"error": "Invalid JSON response"}

def fetch_product_data(product_id: int) -> Dict:
    """Fetch product data from API."""
    try:
        url = f"https://api.example.com/products/{product_id}"
        headers = {
            "Authorization": "Bearer token",
            "Content-Type": "application/json"
        }
        
        response = requests.get(url, headers=headers, timeout=30)
        
        if response.status_code == 200:
            return response.json()
        elif response.status_code == 404:
            return {"error": "Product not found"}
        else:
            return {"error": f"API request failed with status {response.status_code}"}
            
    except requests.RequestException as e:
        return {"error": f"Request failed: {str(e)}"}
    except json.JSONDecodeError:
        return {"error": "Invalid JSON response"}

def fetch_order_data(order_id: int) -> Dict:
    """Fetch order data from API."""
    try:
        url = f"https://api.example.com/orders/{order_id}"
        headers = {
            "Authorization": "Bearer token",
            "Content-Type": "application/json"
        }
        
        response = requests.get(url, headers=headers, timeout=30)
        
        if response.status_code == 200:
            return response.json()
        elif response.status_code == 404:
            return {"error": "Order not found"}
        else:
            return {"error": f"API request failed with status {response.status_code}"}
            
    except requests.RequestException as e:
        return {"error": f"Request failed: {str(e)}"}
    except json.JSONDecodeError:
        return {"error": "Invalid JSON response"}

# Duplicate Pattern 4: Data processing
def process_user_data(raw_data: Dict) -> Dict:
    """Process raw user data."""
    processed = {}
    
    # Extract and clean basic fields
    processed["id"] = raw_data.get("id", 0)
    processed["name"] = raw_data.get("name", "").strip()
    processed["email"] = raw_data.get("email", "").lower().strip()
    
    # Format dates
    if "created_at" in raw_data:
        processed["created_at"] = datetime.fromisoformat(raw_data["created_at"])
    
    if "updated_at" in raw_data:
        processed["updated_at"] = datetime.fromisoformat(raw_data["updated_at"])
    
    # Add computed fields
    processed["display_name"] = processed["name"].title() if processed["name"] else "Unknown User"
    processed["is_active"] = raw_data.get("status", "inactive") == "active"
    
    return processed

def process_customer_data(raw_data: Dict) -> Dict:
    """Process raw customer data."""
    processed = {}
    
    # Extract and clean basic fields
    processed["id"] = raw_data.get("id", 0)
    processed["name"] = raw_data.get("name", "").strip()
    processed["email"] = raw_data.get("email", "").lower().strip()
    
    # Format dates
    if "created_at" in raw_data:
        processed["created_at"] = datetime.fromisoformat(raw_data["created_at"])
    
    if "updated_at" in raw_data:
        processed["updated_at"] = datetime.fromisoformat(raw_data["updated_at"])
    
    # Add computed fields
    processed["display_name"] = processed["name"].title() if processed["name"] else "Unknown User"
    processed["is_active"] = raw_data.get("status", "inactive") == "active"
    
    return processed

def process_member_data(raw_data: Dict) -> Dict:
    """Process raw member data."""
    processed = {}
    
    # Extract and clean basic fields
    processed["id"] = raw_data.get("id", 0)
    processed["name"] = raw_data.get("name", "").strip()
    processed["email"] = raw_data.get("email", "").lower().strip()
    
    # Format dates
    if "created_at" in raw_data:
        processed["created_at"] = datetime.fromisoformat(raw_data["created_at"])
    
    if "updated_at" in raw_data:
        processed["updated_at"] = datetime.fromisoformat(raw_data["updated_at"])
    
    # Add computed fields
    processed["display_name"] = processed["name"].title() if processed["name"] else "Unknown User"
    processed["is_active"] = raw_data.get("status", "inactive") == "active"
    
    return processed

# Duplicate Pattern 5: Logging functions
def log_user_action(user_id: int, action: str, details: str = ""):
    """Log user action."""
    timestamp = datetime.now().isoformat()
    log_entry = {
        "timestamp": timestamp,
        "type": "user_action",
        "user_id": user_id,
        "action": action,
        "details": details
    }
    
    # Write to log file
    with open("user_actions.log", "a") as f:
        f.write(json.dumps(log_entry) + "\n")
    
    # Also print to console for debugging
    print(f"[{timestamp}] User {user_id} performed action: {action}")

def log_admin_action(admin_id: int, action: str, details: str = ""):
    """Log admin action."""
    timestamp = datetime.now().isoformat()
    log_entry = {
        "timestamp": timestamp,
        "type": "admin_action",
        "admin_id": admin_id,
        "action": action,
        "details": details
    }
    
    # Write to log file
    with open("admin_actions.log", "a") as f:
        f.write(json.dumps(log_entry) + "\n")
    
    # Also print to console for debugging
    print(f"[{timestamp}] Admin {admin_id} performed action: {action}")

def log_system_action(system: str, action: str, details: str = ""):
    """Log system action."""
    timestamp = datetime.now().isoformat()
    log_entry = {
        "timestamp": timestamp,
        "type": "system_action",
        "system": system,
        "action": action,
        "details": details
    }
    
    # Write to log file
    with open("system_actions.log", "a") as f:
        f.write(json.dumps(log_entry) + "\n")
    
    # Also print to console for debugging
    print(f"[{timestamp}] System {system} performed action: {action}")

# Duplicate Pattern 6: Cache operations
class UserCache:
    def __init__(self):
        self.cache = {}
        self.max_size = 1000
        self.ttl = 3600  # 1 hour
    
    def get(self, key: str) -> Optional[Dict]:
        """Get item from cache."""
        if key in self.cache:
            entry = self.cache[key]
            if datetime.now().timestamp() - entry["timestamp"] < self.ttl:
                return entry["data"]
            else:
                del self.cache[key]
        return None
    
    def set(self, key: str, data: Dict) -> None:
        """Set item in cache."""
        if len(self.cache) >= self.max_size:
            # Remove oldest entry
            oldest_key = min(self.cache.keys(), 
                           key=lambda k: self.cache[k]["timestamp"])
            del self.cache[oldest_key]
        
        self.cache[key] = {
            "data": data,
            "timestamp": datetime.now().timestamp()
        }

class ProductCache:
    def __init__(self):
        self.cache = {}
        self.max_size = 1000
        self.ttl = 3600  # 1 hour
    
    def get(self, key: str) -> Optional[Dict]:
        """Get item from cache."""
        if key in self.cache:
            entry = self.cache[key]
            if datetime.now().timestamp() - entry["timestamp"] < self.ttl:
                return entry["data"]
            else:
                del self.cache[key]
        return None
    
    def set(self, key: str, data: Dict) -> None:
        """Set item in cache."""
        if len(self.cache) >= self.max_size:
            # Remove oldest entry
            oldest_key = min(self.cache.keys(), 
                           key=lambda k: self.cache[k]["timestamp"])
            del self.cache[oldest_key]
        
        self.cache[key] = {
            "data": data,
            "timestamp": datetime.now().timestamp()
        }

class OrderCache:
    def __init__(self):
        self.cache = {}
        self.max_size = 1000
        self.ttl = 3600  # 1 hour
    
    def get(self, key: str) -> Optional[Dict]:
        """Get item from cache."""
        if key in self.cache:
            entry = self.cache[key]
            if datetime.now().timestamp() - entry["timestamp"] < self.ttl:
                return entry["data"]
            else:
                del self.cache[key]
        return None
    
    def set(self, key: str, data: Dict) -> None:
        """Set item in cache."""
        if len(self.cache) >= self.max_size:
            # Remove oldest entry
            oldest_key = min(self.cache.keys(), 
                           key=lambda k: self.cache[k]["timestamp"])
            del self.cache[oldest_key]
        
        self.cache[key] = {
            "data": data,
            "timestamp": datetime.now().timestamp()
        }

# Exact duplicate functions (intentional)
def calculate_tax_amount(amount: float, tax_rate: float) -> float:
    """Calculate tax amount."""
    return amount * tax_rate

def compute_tax_amount(amount: float, tax_rate: float) -> float:
    """Calculate tax amount."""
    return amount * tax_rate

# Near-duplicate functions with slight variations
def format_user_name(first_name: str, last_name: str) -> str:
    """Format user name."""
    if not first_name and not last_name:
        return "Unknown User"
    elif not first_name:
        return last_name.title()
    elif not last_name:
        return first_name.title()
    else:
        return f"{first_name.title()} {last_name.title()}"

def format_customer_name(first_name: str, last_name: str) -> str:
    """Format customer name."""
    if not first_name and not last_name:
        return "Unknown Customer"
    elif not first_name:
        return last_name.title()
    elif not last_name:
        return first_name.title()
    else:
        return f"{first_name.title()} {last_name.title()}"

def format_employee_name(first_name: str, last_name: str) -> str:
    """Format employee name."""
    if not first_name and not last_name:
        return "Unknown Employee"
    elif not first_name:
        return last_name.title()
    elif not last_name:
        return first_name.title()
    else:
        return f"{first_name.title()} {last_name.title()}"

if __name__ == "__main__":
    # Test the duplicate functions
    print("Testing duplicate code patterns...")
    
    # Test validation functions
    user_errors = validate_user_input("jo", "invalid-email", "123")
    admin_errors = validate_admin_input("jo", "invalid-email", "123")
    print(f"User validation errors: {user_errors}")
    print(f"Admin validation errors: {admin_errors}")
    
    # Test database connections
    user_db = UserDatabase()
    product_db = ProductDatabase()
    user_db.connect()
    product_db.connect()
    
    # Test name formatting
    user_name = format_user_name("john", "doe")
    customer_name = format_customer_name("jane", "smith")
    print(f"User name: {user_name}")
    print(f"Customer name: {customer_name}")
    
    print("Duplicate code test completed.") 