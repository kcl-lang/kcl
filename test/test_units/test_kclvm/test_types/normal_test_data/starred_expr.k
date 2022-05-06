schema Person:
    name: str

a = [1, 2, 3]
b: [int] = [*a, 4, 5]
c = {key1: "value1"}
d: {str:str} = {**c, key2: "value2"}
e: [str] = [*d]
f: [any] = [*Undefined]
g: [any] = [*None]
h: {str:} = {**Undefined, key: "value"}
i: {str:} = {**None, key: "value"}
j: {str:} | Person = {name: "name"}
k: {str:} = {**j}
