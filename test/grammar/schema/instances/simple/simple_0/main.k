schema Person:
    name?: str
    age?: int

person1 = Person {
    name: "Alice"
    age: 18
}

person2 = Person {
    name: "Bob"
    age: 16
}

instances = Person.instances()
