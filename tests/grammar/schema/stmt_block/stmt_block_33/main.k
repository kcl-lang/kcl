schema Name:
    first: str
    last?: str
    age?: int

schema A:
    name: Name

schema Data:
    _alice = A {
        name: {
            first: "aa"
            age: 1
        }
    }

    _alice.name.age += 1
    alice = _alice

data = Data {}
