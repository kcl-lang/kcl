schema Person:
    firstName?: str = None
    lastName?: str = None

schema Student(Person):
    firstName?: str = "John"

student = Student {}
