import json

schema Person:
    name: str
    age: int

persons: [Person] = json.decode('[{"name": "Alice", "age": 18}, {"err_name": "Bob", "age": 10}]')
