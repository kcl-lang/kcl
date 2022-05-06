schema Person:
    name: str = "rename"
    gender: str = "overwrite"

alice = Person {
    "name" = "alice",
    "gender" = "female"
    fullName = "full name"
}
