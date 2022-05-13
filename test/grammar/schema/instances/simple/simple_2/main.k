schema Person:
    name?: str = None
    age?: int = None

_persons = [Person {
    name: "Alice"
    age: 18
}, Person {
    name: "Bob"
    age: 16
}]

instancesLen1 = len(Person.instances())

person = Person {}

instancesLen2 = len(Person.instances())
