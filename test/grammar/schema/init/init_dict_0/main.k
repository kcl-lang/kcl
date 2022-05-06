schema Name:
    name: str

schema Person:
    name: Name = {
        "name": "default name"
    }
    gender: str = "default gender"

alice = Person {
    "name" =  {
        "name": "alice"
    },
    "gender" = "female"
}
