schema PersonMixin:
    person: Person {
        name = "Alice"
    }

schema Person:
    name?: str
    age?: int
    check:
        age > 0 if age

schema Data:
    mixin [PersonMixin]

    person: Person {
        age = 1
    }

data = Data()
