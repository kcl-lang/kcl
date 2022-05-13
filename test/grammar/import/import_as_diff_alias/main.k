import name_pkg.v1 as name
import person_pkg.v1 as person

schema FullName(name.Name):
    fullName: str = "{} {}".format(firstName, lastName)


johnName = name.Name {
    "firstName": "John",
    "lastName": "Snow"
}

johnPerson = person.Person {
    "name": {
        "firstName": "John",
        "lastName": "Snow"
    },
    "age": 10
}

johnFullName = FullName {
    "firstName": "John",
    "lastName": "Snow"
}
