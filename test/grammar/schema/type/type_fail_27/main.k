schema Person:
    name: str = "kcl"
    age: int = 1
    config?: [str]

x0 = Person {}
x1: any = x0
x2 = Person {
    config = x1
}
