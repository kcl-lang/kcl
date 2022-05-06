import pkg

schema Person:
	name: pkg.Name
	age: int

alice = Person {
    "name": {
        "firstName": "alice",
        "lastName": "smith"
    },
    "age": 10
}