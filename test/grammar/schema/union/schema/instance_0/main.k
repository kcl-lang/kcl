schema Name:
    firstName?: str
    lastName?: str

schema Person:
    name: Name = {
        "firstName": "firstName",
        "lastName": "lastName"
    }
    age: int = 0
    info: {str:str} = {
        "card": "123"
    }

schema Boy(Person):
    name: Name |= {
        "firstName" = "John"
    }
    age: int = 18
    info: {str:str} |= {
        "phoneNum": "123"
    }

JohnDoe = Boy {
    "name": {
        "lastName" = "Dow"
    }
}
