schema Person:
    firstName: str = "John"
    lastName: str
    _tmp = "tmp variable"
    lastName = _tmp

john = Person {}
