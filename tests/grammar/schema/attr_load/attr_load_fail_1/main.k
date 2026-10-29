schema Data:
    name: str = "data"

_any: any = Data {}
value = _any.get("name")
