schema Name:
    name: str = "Kiki"
    age: int = 10

schema Name0:
    name0: Name

schema Person:
    name: Name0

John = Person{
    name.name0.name = 123
}
