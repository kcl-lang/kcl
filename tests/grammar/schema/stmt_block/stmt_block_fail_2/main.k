schema Person:
    firstName: str = "John"
    lastName: str
    name: str
    sonName: str = None

    if sonName + "123" == "123":
        name = sonName

schema Son(Person):
    sonName: str = None

JohnDoe = Son {
    "lastName": "Doe",
}
