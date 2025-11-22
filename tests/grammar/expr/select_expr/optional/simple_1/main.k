schema Person:
    name: str

person = Person {name: "Alice"}
data = {}
name1 = person?.name
name2 = name1?.lower()?.upper()[:3] or ""
