
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
_filename = "out.json"
print("JSON output is in {}".format(_filename))
json.dump_to_file(_person, _filename, indent=4, ignore_private=True, ignore_none=True)
