schema PersonMixin:
    info: str = "name: ${name}, age: ${age}"

schema Person:
    mixin [PersonMixin]

    name?: str
    age?: int
    check:
        age > 0 if age

schema Data:
    person: Person {
        age = 1
    }
    person: Person {
        name = "Alice"
    }

data = Data()
