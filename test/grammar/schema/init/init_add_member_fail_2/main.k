schema Name:
    name: str

schema Person:
    gender: str = "overwrite"
    name: Name = {
        "name": "rename",
    }

alice = Person {
    "name" = {
        "name": "alice"
    },
    "gender" = "female"
    error = "CannotAddMembers"
}
