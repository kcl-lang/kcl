import runtime

schema Person:
    age: int
    name?: str

    check:
        0 < age < 10, "age must be in the range (0, 10)"

msg = runtime.catch(lambda {
    person = Person {
        age: 1000
    }
    person.name
})
