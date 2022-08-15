_alice = A {
    name: {
        first: "aa"
        age: 1
    }
}

_alice.name.age = _alice.name.age + 1
_alice.name.age += 1
alice = _alice

schema Name:
    first?: str
    last?: str
    age?: int

schema A:
    name: Name
