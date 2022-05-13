schema PersonProtocol:
    name: str

mixin PersonMixin for PersonProtocol:
    name = 'PersonMixin-name'

schema Person:
    mixin [PersonMixin]
    name: str = "kcl"
    age: int = 1

x0 = Person{}
x1 = Person{age = 101}
