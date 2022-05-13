_dict = {"key": "value"}
v1 = _dict["keyNotExist"] or None
v2 = _dict[None] or None
v3 = _dict[Undefined] or None
