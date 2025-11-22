schema Data:
    id: int = 1

schema Person:
    name: str = "kcl"
    age: int = 1
    data: Data = Data {}

x0 = Person {}
x1 = Person {
    age = 101
}
x2 = Person.instances()
x3 = Data.instances()
