schema Person:
    name: str
    age: int
    tagged: bool

_name = option("name")
_age = int(option("age"))
_tagged = bool(option("tagged"))

alice = Person {
    "name": _name,
    "age": _age,
    "tagged": _tagged
}
