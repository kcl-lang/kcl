@deprecated
schema Person:
    firstName: str = "John"
    lastName: str
    name: str

schema Son(Person):
    name: str = "Son"

JohnDoe = Son {}
