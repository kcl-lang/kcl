import datetime

_t = datetime.today().split(".")
assert len(_t) == 2
result: bool = datetime.validate(_t[0], "%Y-%m-%d %H:%M:%S")
