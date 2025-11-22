
import yaml

schema Person:
    name?: str
    age?: int
    school?: str
    data?: [int] = [1, 2, None]

_person = Person {
    name: "Alice"
    age: 18
}
a1 = yaml.encode(_person, ignore_private=True)
a2 = yaml.encode(_person, ignore_private=True, ignore_none=True)
