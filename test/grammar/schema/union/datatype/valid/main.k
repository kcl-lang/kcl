schema Person:
    firstName: str = "firstName"
    lastName: str = "lastName"
    age: int = 0

schema Boy(Person):
    firstName: str = "John"
    lastName: str = "Doe"
    age: int |= 18

JohnDoe = Boy {}
