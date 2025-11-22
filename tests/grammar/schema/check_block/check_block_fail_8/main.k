schema Person:
    labels: [str]
    name: str

    check:
        name, "name should be defined and not empty"
        labels, "labels should be defined and not empty"

JohnDoe = Person {
    "name": "John",
    "labels": []
}
