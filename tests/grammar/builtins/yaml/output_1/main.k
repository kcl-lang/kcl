
import yaml

schema Person:
    name?: str
    age?: int
    school?: str
    data?: [int] = [1, 2, None]

person = Person {
    name: "Alice"
    age: 18
}
filename = "out.yaml"
yaml.dump_to_file(person, filename, ignore_private=True, ignore_none=True)
