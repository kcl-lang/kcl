schema Person[separator]:
    firstName: str = "John"
    lastName: str
    fullName: str = firstName + separator + lastName

x = Person(separator=" ") {lastName = "Doe"}

y = Person("-") {lastName = "Doe1"}

z = x | y
