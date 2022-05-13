schema Person:
    name: str
    age: int

key = "age"
_alice = Person {
    name: "Alice"
}
_alice = Person {"${key}": 18}
alice = _alice
