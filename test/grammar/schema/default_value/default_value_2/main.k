schema Person:
    firstName : str = "John"
    lastName : str
    dict: {str:} = {}

JoeDoe = Person {
    "firstName" = "Joe",
    "lastName" = "Doe",
    "dict" = {"key": "value", "key2": 1}
}
