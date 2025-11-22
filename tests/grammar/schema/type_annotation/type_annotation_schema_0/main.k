schema Person:
    name: str = "kcl"
    age: int = 1

x0: Person = Person {}
x1: Person = {
    age = 101
}
