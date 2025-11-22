schema Person:
    family: [{str:str}]
    name: str
    names: [str] = [people["name"] for people in family]

    check:
        len([True for people in family if "name" in people]) == len(family), "all family should have name defined."

alice = Person {
    name: "alice",
    family: [
        {
            name: "bob"
        },
        {
            name: "john"
        }
    ]
}
