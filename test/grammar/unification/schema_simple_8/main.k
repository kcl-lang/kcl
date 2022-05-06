schema PersonFooMixin:
    person: Person {
        if appendInfo:
            info.key2: "value2"
            name = "Alice"
    }

schema PersonBarMixin:
    person: Person {
        if appendInfo:
            info.key3: "value3"
    }

schema Person:
    name?: str
    age?: int
    info?: {str:}
    check:
        age > 0 if age

schema Data[appendInfoVar: bool]:
    mixin [
        PersonFooMixin,
        PersonBarMixin
    ]

    appendInfo: bool = appendInfoVar
    person: Person {
        if appendInfo:
            info.key1: "value1"
            age = 1
    }

dataWithInfo = Data(True)
dataWithoutInfo = Data(False)
