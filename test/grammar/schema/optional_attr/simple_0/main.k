schema Person:
    name: str
    age?: int = None

person1 = Person {
    name: "Alice"
}
person2 = Person {
    name: "Bob"
    age: 10
}
