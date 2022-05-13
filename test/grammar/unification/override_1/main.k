schema Person:
    name: str
    age: int
    env: [{str:}]

alice = Person {
    name: "Alice"
    env: [{
        name: "key1"
        value: "value1"
    }, {
        name: "key2"
        value: "value2"
    }]
}

alice = Person {
    age: 18
    env = [{
        name: "key2"
        value: "value2"
    }]
}
