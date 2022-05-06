schema Person:
    name: str = "Alice"
    age: int = 18

person = Person {}
count = Person.err_instances()
