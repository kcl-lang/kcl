schema Person:
    name?: str
    age?: int

_persons = [Person {
    name: "Alice"
    age: 18
}, Person {
    name: "Bob"
    age: 16
}]

instances = Person.instances()
