schema Name0:
    name: Name1

schema Name1:
    name: str

schema Person:
    name: Name0 = Name0 {
        "name": Name1 {
            "name": "default name"
        }
    }
    gender: str = "default gender"

alice = Person {
    "name" = {
        "name": {
            "name": "alice"
        }
    },
    "gender" = "female"
}
