schema Person:
    name: str

person = Person {name: "Alice"}
data = {"name": {"key": "value"}}
name1 = person?.name?.lower()  # Alice
name2 = data["name"]["key"]?.upper()?.lower()
datalist = [[1], 2, 3]
item = list(datalist?[0])?[0]
name3 = data?["name"]?["err_key"]?.lower() or data?["name"]?["err_key"]?.upper() or "name3_value"
