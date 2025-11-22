_alice = A {
    _name: {
        first: "aa"
    }
}

_alice._name |= {last: "value"}

schema Name:
    first: str
    last?: str

schema A:
    _name: Name
