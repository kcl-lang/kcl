schema Person:
    name: str
    age: int
    hc: [int]

alice = Person {
    name: "Alice"
    hc: [1, 2]
}

alice = Person {
    age: 18
    hc = [2]
}

name = alice.name
age = alice.age
hc = alice.hc
