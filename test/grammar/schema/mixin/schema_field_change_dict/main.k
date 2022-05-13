schema Person:
    firstName: str
    lastName: str
    fullName: str
    info: {str:str}

schema InfoMixin:
    info |= {"phone": "321"}

schema Scholar(Person):
    mixin [InfoMixin]
    school?: str = None

JohnDoe = Scholar {
    "firstName": "John",
    "lastName": "Doe",
    "fullName": "Doe Jon",
    "info": {
        "id": "123"
    }
}
