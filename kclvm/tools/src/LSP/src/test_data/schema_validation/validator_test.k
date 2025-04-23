# This file contains test cases for schema validation
# Each instance below tests different validation scenarios

schema Person:
    name: str  # required
    age: int   # required
    address?: str  # optional

# Test case 1: Valid instance - all required attributes present
person1 = Person {
    name: "John"
    age: 30
    address: "123 Main St"
}

# Test case 2: Invalid instance - missing required 'age' attribute
# Expected error: Missing required attributes in Person instance: age
# Expected error line: 20
person2 = Person {
    name: "Jane"
}

# Test case 3: Invalid instance - missing required 'name' attribute
# Expected error: Missing required attributes in Person instance: name
# Expected error line: 26
person3 = Person {
    age: 25
}

# Test case 4: Valid instance - optional attribute omitted
person4 = Person {
    name: "Bob"
    age: 40
} 