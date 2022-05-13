schema Person:
    name: str
    gender: str

    check:
        name, "name of person should be defined and not empty"
        gender, "gender of person should be defined and not empty"

schema Girl(Person):
    gender: str = "female"

alice = Girl {
    "name": "alice"
}
