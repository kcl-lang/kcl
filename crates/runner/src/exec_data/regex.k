import regex

schema Data:
    name: str = "name"

    check:
        regex.match(name, "name")

data = Data {}
