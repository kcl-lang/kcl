import math

name = "Alice"
schema Base:
    hc: int = 3
    key: str = name

schema Person(Base):
    name: str = "Alice"
    age: int = 18
    labels: {str:str} = {"key": "value", "ageLabel": "ageVal " + str(age)}
    info: [int|str] = [name, age]

person1 = Person {
    "name" = "Bob"
    "age" = 16
}
ceil_val = math.ceil(1.1)
a = 1.1 + 1.1 + 6.6 + 3.2 + abs(-1.2)
b = {"key1": "value1", "key2": "value2"}
attr = b.key1 + person1.name
c = 3 - 2 * 3 / 4
d = a + 2
e = a + 2
f = "ABC" + "234{}"
p = "ABC"[::-1]
ff = f.format("123")
fff = ff.lower()
q = "1" * 12
g = True
l = [1, 2, 3, attr + "value1"]
ll = [*l, 1]
_e = 1 + 1
aug1 = aug2 = 4 + 3
data = [1, 2, 3, 4]
lcomp = [_d * 2 for _d in data for _d in data]
dcomp = {dd: dd * 2 for dd in data}
data0 = data[0]
data12 = data[::-1]
pk = "ABC"[::-1]
qk = [1, 2, 3][::-1]
dict_data = {**person1, **{"key" = "value"}, "key2": "value2"}
