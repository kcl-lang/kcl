schema Person:
    mixin [SumMixin]
    x: int
    y: int
    z: int

schema SumMixin:
    z: int = x + y

person1 = Person {
    "x" = 1
    "y" = 2
}
data = {
    "x": 2
    **person1
}
person2 = Person {
    **data
}
