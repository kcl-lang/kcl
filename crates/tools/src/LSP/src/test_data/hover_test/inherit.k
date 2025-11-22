schema Data:
    name: str = "1"
    age: int

schema Data1[m: {str:str}](Data):
    name = m.name
    