
import json

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
print("[", end="")
print(json.encode(_person, indent=4), end=",\n")
print(json.encode(_person, indent=4, ignore_private=True), end=",\n")
print(json.encode(_person, indent=4, ignore_none=True), end=",\n")
print(json.encode(_person, indent=4, ignore_private=True, ignore_none=True), end="]\n")
