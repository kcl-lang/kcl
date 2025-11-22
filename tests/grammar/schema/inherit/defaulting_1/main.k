schema Person:
    firstName?: str = "firstName"
    lastName?: str = None

schema Student(Person):
    firstName?: str = None

student = Student {}
