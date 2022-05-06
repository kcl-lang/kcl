schema Person:
    firstName: str = "John"
    lastName: str = "Doe"

    headcount: [int] = [i for i in [1, 2, 3]]

    _dict: {str:} = {"key1": "value1", "key2": "value2"}
    keyValue: {str:} = {key: _dict[key] for key in _dict}

JohnDoe = Person {}
