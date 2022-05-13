schema Person:
    name: str
    age: int
    
_person = Person {
    name: "Alice"
    age: 18
}
_person = _person | {
    err: "123"
    age: 10
}
