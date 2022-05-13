
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
_filename = "out.yaml"
print("YAML output is in {}".format(_filename))
yaml.dump_to_file(_person, _filename, ignore_private=True, ignore_none=True)
