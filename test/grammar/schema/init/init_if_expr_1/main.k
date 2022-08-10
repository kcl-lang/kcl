schema Person:
    mixin [InfoMixin]
    name?: str
    age?: int = None
    info?: {str: int|str}

schema InfoMixin:
    info = {"age": age} if age else {}

alice = Person {
    "name": "alice",
    "age": 10,
    "info": {"gender": "girl"}
}

John = Person {
    "name": "john",
    "info": {"gender": "boy"}
}
