schema Person:
    name: str
    age: int
    gender: str
    info: str = "{}, {}, {} years old".format(name, gender, age)

alice = Person {
    "name": "alice",
    "age": 10,
    "gender": "female"
}
