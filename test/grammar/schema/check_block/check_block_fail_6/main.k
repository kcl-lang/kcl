schema Person:
    labels?: {str:str}
    name?: str

    check:
        name, "name should be defined and not empty"
        labels, "labels should be defined and not empty"

JohnDoe = Person {
    "name": "John"
}
