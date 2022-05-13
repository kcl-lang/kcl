
import yaml

schema Person:
    __settings__: {str:str} = {"output_type": "STANDALONE"}
    name?: str
    age?: int
    school?: str
    data?: [int] = [1, 2, None]

_person = Person {
    name: "Alice"
    age: 18
}
# print(yaml.encode(_person), end="")
# print("---")
# print(yaml.encode(_person, ignore_private=True), end="")
# print("---")
# print(yaml.encode(_person, ignore_none=True), end="")
# print("---")
# print(yaml.encode(_person, ignore_private=True, ignore_none=True), end="")

a1 = yaml.encode(_person, ignore_private=True)
a2 = yaml.encode(_person, ignore_private=True, ignore_none=True)
