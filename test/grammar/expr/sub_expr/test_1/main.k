lists = ["Alice", "Bob", "Green"]
name = lists[0].upper()[0:3]
dicts = {"key": "value + {}"}
val = dicts["key"].lower().format(str(lists[0].upper().lower()[0:3]).upper().count('A'))
val_temp = str(lists[1]).upper()
