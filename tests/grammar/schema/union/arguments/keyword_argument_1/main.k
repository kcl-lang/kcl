schema Person[separator]:
    firstName: str = "John"
    lastName: str
    fullName: str = firstName + separator + lastName

x = Person(" ") {lastName = "Doe"}

y = Person(separator="-") {lastName = "Doe1"}

z = x | y
