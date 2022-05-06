schema Person:
    '''Person is a simple schema

    Attributes
    ----------
    name: str, default is "Default"
        A Normal attribute named 'name'
    age: int, default = 18, optional
        A Normal attribute named 'age'

    Examples
    --------
    person = Person {
        name: "Alice"
        age: 18
    }
    '''
    name: str
    age?: int

person = Person {}
