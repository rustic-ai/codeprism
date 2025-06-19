import os
from typing import List, Optional
import json as j

class Calculator:
    """A simple calculator class."""
    
    def __init__(self, initial_value: float = 0.0):
        self.value = initial_value
        self.history = []
    
    def add(self, number: float) -> float:
        """Add a number to the current value."""
        self.value += number
        self.history.append(f"Added {number}")
        return self.value
    
    def subtract(self, number: float) -> float:
        """Subtract a number from the current value."""
        self.value -= number
        self.history.append(f"Subtracted {number}")
        return self.value
    
    def get_history(self) -> List[str]:
        """Get the calculation history."""
        return self.history.copy()
    
    @property
    def current_value(self) -> float:
        """Get the current value."""
        return self.value
    
    @staticmethod
    def multiply_static(a: float, b: float) -> float:
        """Static method to multiply two numbers."""
        return a * b

def create_calculator() -> Calculator:
    """Factory function to create a calculator."""
    calc = Calculator()
    return calc

# Usage
calc = create_calculator()
result = calc.add(10)
final_result = calc.subtract(3)
history = calc.get_history()

# Use static method
product = Calculator.multiply_static(5, 4) 