schema Person:
    firstName : str
    cards : {int:{int:}}

JohnDoe = Person {
    "firstName": "John",
    "cards": {1:{"error":"error"}}
}
