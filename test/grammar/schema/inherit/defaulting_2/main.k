schema Person:
    firstName?: str = "firstName"
    lastName?: str = None

schema Student(Person):
    firstName?: str

student = Student {
    "firstName" = "John"
}
