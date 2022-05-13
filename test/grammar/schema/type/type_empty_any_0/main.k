schema Base:
    name: str

schema Person:
    info: {:} = {"key": []}
    info: {:} = {"key": {}}
    info: {:} = {"key": [1, 2, 3]}
    info: {:} = {"key": [[]]}
    info: {:} = {"key": [[], {}]}
    info: {:} = {"key": "value"}

person = Person {}
