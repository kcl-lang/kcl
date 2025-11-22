name = "Alice"
schema Base:
    hc: int = 3
    key: str = name

    check:
        hc > 0

schema Person(Base):
    name: str = "Alice"
    age: int = 18
    labels: {str:str} = {"key": "value", "ageLabel": "ageVal " + str(age)}
    info: [int|str] = [name, age]

person1 = Person {
    "name" = "Bob"
    "age" = 16
}

person2 = Person()

person3 = person1 {
    name = "Alice"
    age = 18
}

schema InfoMixin:
    info |= {"age": age} if age else {}

schema Info:
    mixin [InfoMixin]
    name?: str
    age?: int = None
    info?: {str: int|str}

info = Info {
    name: "alice"
    age: 10
    info.gender: "girl"
}
