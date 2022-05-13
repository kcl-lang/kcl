"""Type annotation test"""

import pkg
import math
import pkg

schema PersonMixin:
    nameUpper: str = name.upper()

schema Person(Base):
    # Mixin
    mixin [PersonMixin]
    # Index signature
    [str]: int|str|{str:}
    # Attributes
    name: str = "Alice"
    
    _age: int = 18
    _age += 10
    labels: {str:} = {key1: "value1"}
    lebels: {str:} |= {key2: "value2"}
    # Check block
    check:
        _age > 0

@deprecated
schema Base(pkg.Person):
    data?: str

print("Hello type annotation check")

a: int = 1
b: float = 1.0
c: float = 1
d: bool = True
e: bool = False
f: bool = None
g: bool = Undefined
h: [str] = ["Red", "Yellow", "Blue"]
i: [] = ["Red", "Yellow", "Blue"]
j: ["Red"|"Yellow"|"Blue"] = ["Red", "Yellow", "Blue"]
k: [int|str] = [1, 2, "s", "ss"]
l: [{str:}] = [{key: "value"}, {key.key1.key2: "value"}]
m: {str:str|int} = {"key1": "value1", "key2": 2}
n: "Red"|"Yellow"|"Blue" = "Red"
o: Person = Person {
    name: "Bob"
    age: 10
}
p: pkg.Person = pkg.Person {
    name: "Alice"
    age: 18
}
q: Person = {
    name: "Bob"
    age: 18
}
r: Base = Person {
    name: "Bob"
    age: 18
}
s: any = 1
t: any = "s"
u: any = (r + s)
v: str = p.name
w: int = q?.age
x: int|str = 1 if True else "s"
y: [int] = [i ** 2 for i in [1, 2, 3]]
z: {str:} = {"${k}": int(k) for k in ["1", "2", "3"]}
