schema Name1:
    name: str

schema Name2:
    name: str
    age: int

schema Name3:
    [...str]: str
    name: str
    age: int

a: Name1 = {name: ""}
b: Name1 | Name2 = {
    name: "a"
    age: 1
}
c: Name1 | Name2 | Name3 = {
    name: "a"
    age: 1
    "c": "c"
}

d: Name1 | Name2 | {str:Name1} | {str:Name2} = {
    "b": {
        name: "a"
        age: 1
    }
}

e: Name1 | Name2 | {str:Name1} | {str:Name2} | {str: Name3} = {
    "b": {
        name: "a"
        age: 1
        "c": "c"
    }
}
