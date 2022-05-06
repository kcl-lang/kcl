schema Person:
    first: str
    last: str
    age: int

schema Girl:
    gender: str = "female"

alice = Girl {
    "first": "alice",
    "last": " Green",
    "age": 10
}
