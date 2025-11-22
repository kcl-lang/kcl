schema Person:
    firstName?: str = "John"
    lastName?: str = None

schema Student(Person):
    firstName?: str

student = Student {}
