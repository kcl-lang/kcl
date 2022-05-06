
import yaml

_person = {
    "_key": "value"
    "name": "Alice"
    "age": 18
    "data": [1, 2, None]
    "labels": {
        "key1": "value1"
        "_key2": "value2"
        "key3": None
    }
}
print(yaml.encode(_person), end="")
print("---")
print(yaml.encode(_person, ignore_private=True), end="")
print("---")
print(yaml.encode(_person, ignore_none=True), end="")
print("---")
print(yaml.encode(_person, ignore_private=True, ignore_none=True), end="")
