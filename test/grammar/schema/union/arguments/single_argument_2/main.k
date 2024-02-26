schema Person[separator]:
    firstName: str = "John"
    lastName: str
    fullName: str = firstName + separator + lastName

x = Person(" ") {lastName = "Doe"}

y = Person("-") {lastName = "Doe1"}

z = Person("*") {
    **x
    **y
}
