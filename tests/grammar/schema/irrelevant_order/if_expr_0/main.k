schema Person:
    name: str = _name
    age: int = _age or 18
    _name = "Alice" if True else Undefined
    _age = 10 if False else Undefined

person = Person {
    name = "Overwrite"
}
