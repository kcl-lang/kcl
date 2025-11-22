schema PersonFooMixin:
    person: Person {
        name = "Alice"
        info.key2: "value2"
    }

schema PersonBarMixin:
    person: Person {
        info.key3: "value3"
    }

schema Person:
    name?: str
    age?: int
    info?: {str:}
    check:
        age > 0 if age

schema Data:
    mixin [
        PersonFooMixin,
        PersonBarMixin
    ]

    person: Person {
        info.key1: "value1"
        age = 1
    }

data = Data()
