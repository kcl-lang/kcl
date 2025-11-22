schema Name0:
    name: str

schema Name:
    name: Name0 | str

schema Person:
    name: Name
    age: int

person = Person {
    name: Name {
        name: Name0 {
            name: "Alice"
        }
    }
    age: 18
}
