schema Person:
    firstName?: str = "firstName_person"
    lastName?: str = None

schema Student(Person):
    firstName?: str = "firstName_student"

student = Student {
    "firstName" = "John"
}
