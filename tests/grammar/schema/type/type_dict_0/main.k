
schema Person:
    firstName : str
    cards : {str:}
    cards2 : {:int}
    cards3 : {str:int}
    cards4 : {:}

JohnDoe = Person {
    "firstName": "John",
    "cards": {"1":1, "2":2, "3":3},
    "cards2": {"1":1, "2":2, "3":3},
    "cards3": {"1":1, "2":2, "3":3},
    "cards4": {"1":1, "2":2, "3":3}
}
