
schema Name:
    firstName: str
    lastName: str

    # print("--initiating Name-- name:{} {}".format(firstName, lastName))

schema Person(Name):
    gender: str

    # print("--initiating Person-- name:{} {}, gender:{}".format(firstName, lastName, gender))

alice = Person {
    "firstName": "alice",
    "lastName": "Smith",
    "gender": "female"
}
