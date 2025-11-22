schema Name:
    name: str

schema Person:
    gender: str = "overwrite"
    name: Name = {
        "name": "rename",
        "fullName": "full name"
    }

alice = Person {
    "name" = {
        "name": "alice"
    },
    "gender" = "female"
}
