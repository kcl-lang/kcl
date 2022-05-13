schema Person:
    name: str
    family: {str:str}

    check:
        "father" not in family if family

SameenShaw = Person {
    "name": "Shaw",
    "family": {"father": "Finch"}
}
