schema PersonMixin:
    nameUpper: str = name.upper()
    
schema Person:
    mixin [PersonMixin]
    _temp = 1
    _temp = 2
    name: str = "Alice"
    age: int = 10

name = "Bob"
age = 10
base = {name: "Alice", age: 10}
person = Person {
    **base
    name: name
    age: age
}
