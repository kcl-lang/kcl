schema Person:
    name?: str = None
    family?: {str:str} = None

    check:
        ("father" in family) if family

SameenShaw = Person {
    "name": "Shaw"
}
