schema Person:
    [str]: any
    name: str = "Alice"
    age: int = 10
    data: [int] = [1, 2, 3]
    labels: {str:}

name = ""
age = 1
person = Person {
    name: name
    age: age
    "1": 1
    data[0] += 1
    data[1] = 1
    data += [2]
    "temp": "temp"
    labels.key1: 1
    labels.key2: "1"
}
