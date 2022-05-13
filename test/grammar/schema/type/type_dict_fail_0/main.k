schema Name:
  firstName : str
  lastName: int

schema Person:
  name: Name
  age: int

schema User:
  person: Person
  phone: int

alice = User {
  "person": {
      "name" : {
          "firstName": "Alice",
          "lastName": "Terry"
      },
      "age" : 12
  },
  "phone": 123
}
