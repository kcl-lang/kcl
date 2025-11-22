schema Person:
    name: str
    age: int
    gender: str
    info_with_type: str = "{}, {}, {} years old".format(name, gender, age)
    info_without_type: str = "{}, {}, {} years old".format(name, gender, age)

schema Girl(Person):
    gender: str = "female"

alice = Girl {
    "name": "alice",
    "age": 10
}
