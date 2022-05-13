schema Person:
    firstName?: str = "John"
    lastName?: str = None
    name?: str = None

@deprecated
schema Son(Person):
    name: str = "Son"

JohnDoe = Person {}
