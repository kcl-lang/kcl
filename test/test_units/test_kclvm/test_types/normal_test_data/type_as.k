import pkg

person: pkg.Person = pkg.Person {
    name = "Alice"
    age = 18
}
personOther = person as pkg.Person
