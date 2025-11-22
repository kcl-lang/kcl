schema Person:
    firstName: str = "John"
    lastName: str = "Doe"
    headcount: [int] = []
    keyValue: {str:}

    headcount += [1, 2, 3]
    headcount += [4, 5, 6]

    _keyValue = {"key2": "value2"}
    keyValue = {
        **{"key1": "value1"}
        **keyValue,
        **_keyValue
    }

JohnDoe = Person {}
