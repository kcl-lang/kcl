schema Base:
    hc?: int = None

schema Person(Base):
    name?: str = None
    age?: int = None

_base = Base {
    hc: 3
}

_persons = [Person {
    name: "Alice"
    age: 18
}, Person {
    name: "Bob"
    age: 16
}]

baseInstance = Base.instances()
personInstanceLen = len(Person.instances())
