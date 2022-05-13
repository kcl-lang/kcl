schema Person:
    first: str
    last: str
    age: int

schema Chinese:
    mixin [ChineseMixin]
    first: str
    last: str
    age: int

schema ChineseMixin:
    chinesePerson = Person {
        "last": last,
        "age": age,
        "frist": first
    }

alice = Chinese {
    "first": "alice",
    "age": 10
}
