schema Person:
    name: str = _name
    age: int = _age
    if True:
        _name = "Alice"
    if False:
        _age = 10
    else:
        _age = 18

person = Person {
    name = "Overwrite"
}
