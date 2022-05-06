schema Person[_age, _name]:
    name: str = _name
    age: int = _age

person = Person(12, _name="Alice") {}
