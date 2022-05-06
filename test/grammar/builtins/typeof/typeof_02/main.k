schema Person:
    name: str = "Alice"
    count: int = 18

_persons = [Person {}] + [Person {}]
_persons += [Person {}]
_data = "123" + "456"
_data += "789"
person0_type = typeof(_persons[0])
person1_type = typeof(_persons[1])
person2_type = typeof(_persons[2])
str_type = typeof(_data)
