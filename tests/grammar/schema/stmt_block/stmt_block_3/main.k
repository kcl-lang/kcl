_separator_out_side = '-'

schema Person:
    firstName: str = "John"
    lastName: str
    _separator: str = '+'
    name: str = firstName + _separator + _separator_out_side + lastName

JohnDoe = Person {
    "lastName": "Doe"
}
