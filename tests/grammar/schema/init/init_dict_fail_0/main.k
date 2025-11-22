schema Name:
    name: str

schema Person:
    name: Name = {
        "name": "rename",
        "gender": "female"
    }
    gender: str = "overwrite"

alice = Person {
    "name": {
        "name": "alice"
    },
    "gender": "female"
}
