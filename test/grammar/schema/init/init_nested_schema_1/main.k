schema Name0:
    name: Name1

schema Name1:
    name: str

schema Person:
    name: Name0
    gender: str = "default gender"
    name.name.name = "rename"

alice = Person {
    "name" = {
        "name": {
            "name": "alice"
        }
    },
    "gender" = "female"
}
