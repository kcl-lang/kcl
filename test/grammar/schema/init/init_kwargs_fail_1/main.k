schema Person[_name, _age]:
    name: str = _name
    age: int = _age

person = Person(_name="Alice", 12) {}
