schema Person:
    name: str
    age: int
    tagged: bool

alice = Person {
    "name": option("name"),
    "age": int(option("age")),
    "tagged": bool(option("tagged"))
}
