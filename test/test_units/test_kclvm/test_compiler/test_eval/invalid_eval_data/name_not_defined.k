schema GetData:
    result: str = name

schema Person:
    name: str = "kcl"
    result: GetData = GetData {}

x0 = Person {}
