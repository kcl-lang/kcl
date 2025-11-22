
import json

schema Person:
    name?: str
    age?: int
    school?: str
    data?: [int] = [1, 2, None]

person = Person {
    name: "Alice"
    age: 18
}
filename = "out.json"
json.dump_to_file(person, filename, indent=4, ignore_private=True, ignore_none=True)
