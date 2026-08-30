schema Person:
    name: str
    age: int

name = "next"
age  = 30

p = Person { name, age }
m = { name, age }
n = Person { name, age = 99 }
