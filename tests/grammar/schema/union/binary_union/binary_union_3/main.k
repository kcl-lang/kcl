schema Person:
    name: str
    labels: {str:str}

person1 = Person {
    name = "Alice"
    labels: {
        "key1": "value1"
    }
}
person2 = Person {
    name = "Bob"
    labels: {
        "key2": "value2"
    }
}
person3 = person1 | person2
