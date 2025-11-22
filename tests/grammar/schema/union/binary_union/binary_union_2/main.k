schema Name:
    key1?: str = None
    key2?: str = None

schema Person:
    name1?: Name = {"key1": "value1"}
    name2?: Name = name1 | {"key2": "value2"}

person = Person {}
