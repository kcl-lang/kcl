
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
person0 = json.encode(_person, indent=4)
person1 = json.encode(_person, indent=4, ignore_private=True)
person2 = json.encode(_person, indent=4, ignore_none=True)
person3 = json.encode(_person, indent=4, ignore_private=True, ignore_none=True)
