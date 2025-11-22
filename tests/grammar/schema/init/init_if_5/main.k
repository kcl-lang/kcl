schema Person:
    name?: str = None
    dependOnName?: str = None

    if name:
        dependOnName = "value"

JohnDoe = Person {}
