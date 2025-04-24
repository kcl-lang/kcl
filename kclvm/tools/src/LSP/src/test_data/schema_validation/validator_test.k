# This file contains test cases for schema validation
# Each instance below tests different validation scenarios

schema Person:
    name: str  # required
    age: int   # required
    address: str  # now required for nested validation

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

# Test case 4: Valid instance - all required attributes present
person4 = Person {
    name: "Bob"
    age: 40
    address: "456 Oak St"  # now required
}


schema NestedPerson:
    family: Person  # nested validation up to depth 10

# Test case 5: Valid instance - nested Person instance
nested_person = NestedPerson {
    family: Person {
        name: "John"
        age: 30
        address: "123 Main St"
    }
}

# Test case 6: Invalid instance - nested Person missing required 'address' attribute
# Expected error: Missing required attributes in nested Person instance: address
# Expected error line: 55
nested_person3 = NestedPerson {
    family: Person {
        name: "John"
        age: 30
        # address is required but missing
    }
}

# Test case 7: Invalid instance - missing required 'age' attribute
# Expected error: Missing required attributes in Person instance: age
# Expected error line: 70
CreateTest = lambda name: str -> NestedPerson {
    NestedPerson {
        family = Person {
            name = name
        }
    }
}

