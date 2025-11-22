schema Person:
    name?: str = "Alice"
    age?: int = None

person1 = Person {}
person2 = Person {
    name = "Bob"
    age = 10
}
