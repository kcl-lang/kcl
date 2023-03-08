schema Parent:
    name: str

schema Son(Parent):
    age: int
    son_name: str = name

schema GrandSon(Son):
    grand_son_name: str = name
    a: str = name_not_exist
