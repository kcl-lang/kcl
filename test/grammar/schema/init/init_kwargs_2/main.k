schema Person[_gender, _age, _name]:
    name: str = _name
    age: int = _age
    gender: str = _gender

person = Person(_gender="male", _name="Bob", _age=12) {}
