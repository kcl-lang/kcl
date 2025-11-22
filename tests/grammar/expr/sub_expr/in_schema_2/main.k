schema Info:  
    lists: [str] = ["Alice", "Bob", "Green"]
    name: str = lists[0].upper()[0:3]
    dicts: {str:} = {"key": "value + {}"}
    val: str = dicts["key"].lower().format(str(lists[0].upper().lower()[0:3]).upper().count('A'))
    val_temp: str = str(lists[1]).upper()

info = Info {}
