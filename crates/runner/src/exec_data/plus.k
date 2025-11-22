schema Person:
    name: str = "Alice"
    count: int = 18

_persons = [Person {}] + [Person {}]
_persons += [Person {}]
_data = "123" + "456"
_data += "789"
persons = _persons
data = _data
