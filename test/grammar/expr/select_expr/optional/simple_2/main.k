schema Person:
    name: str

person = Person {name: "Alice"}
data = {"name": {"key": "value"}}
name1 = person?.name?.lower()  # alice
name2 = data["name"]?["key"]?.upper()
# name3 = data["name"]["err_key"]?.upper()
