schema Person:
    name?: str = None
    family?: {str:str} = None

    check:
        ("father" not in family) if family

SameenShaw = Person {
    "name": "Shaw"
}
