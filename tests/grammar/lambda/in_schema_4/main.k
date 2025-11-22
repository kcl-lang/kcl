schema Person:
    name: str = "kcl"
    age: int = 1
    ageStr: str = (lambda {
        prefix = "age: "
        prefix + str(age)
    })()

x0 = Person {}
x1 = Person {
    age = 101
}
