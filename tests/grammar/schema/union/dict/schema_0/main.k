schema Person:
    name: {str:str} = {
        "firstName": "firstName",
        "lastName": "lastName"
    }
    age: int = 0
    info: {str:str} = {
        "card": "123"
    }

schema Boy(Person):
    name: {str:str} |= {
        "firstName" = "John"
    }
    age: int = 18
    info: {str:str} |= {
        "phoneNum": "123"
    }

JohnDoe = Boy {}
