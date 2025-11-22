schema Person:
    firstName: str
    cards: {str:int}

JohnDoe = Person {
    "firstName": "John",
    "cards": {"error": "error"}
}
