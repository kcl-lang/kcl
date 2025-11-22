schema Person:
    name?: str = None
    dependOnName?: str = "default"

    if name:
        dependOnName = "value"

JohnDoe = Person {}
