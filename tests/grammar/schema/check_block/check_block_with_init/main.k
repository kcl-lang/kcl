
schema Person:
    name: str

    # print("init block do nothing")

    check:
        name not None, "we fail here"

JohnDoe = Person {
    "name": "John"
}
