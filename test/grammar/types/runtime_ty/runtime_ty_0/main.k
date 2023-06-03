import json

schema Person:
    name: str
    age: int

person: Person = json.decode('{"name": "Alice", "age": 18}')
