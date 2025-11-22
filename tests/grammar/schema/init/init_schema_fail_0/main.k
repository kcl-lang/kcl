schema Name:
    name: str

schema Name0:
    name: str

schema Person:
    name: Name = Name0 {
        "name": "default"
    }
    gender: str = "default"

alice = Person {
    "name": {
        "name": "alice"
    },
    "gender": "female"
}
