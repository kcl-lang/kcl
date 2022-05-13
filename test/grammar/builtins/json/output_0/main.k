
import json

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
#print("[", end="")
#print(json.encode(_person, indent=4), end=",\n")
#print(json.encode(_person, indent=4, ignore_private=True), end=",\n")
#print(json.encode(_person, indent=4, ignore_none=True), end=",\n")
#print(json.encode(_person, indent=4, ignore_private=True, ignore_none=True), end="]\n")

a1 = json.encode(_person, indent=4, ignore_private=True)
a2 = json.encode(_person, indent=4, ignore_private=True, ignore_none=True)
