schema Person:
    name: str

person = Person {name: "Alice"}
data = {}
name1 = person?.name
name2 = data["key"]?.name
name3 = name1?.lower() or ""
