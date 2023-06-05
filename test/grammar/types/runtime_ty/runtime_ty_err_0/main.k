import json

schema Person:
    name: str
    age: int

person: Person = json.decode('{"err_name": "Alice", "age": 18}')
