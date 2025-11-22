schema Person:
    firstName: str
    cards: [{str:str}]

JohnDoe = Person {
    "firstName": "John",
    "cards": ["error"]
}
