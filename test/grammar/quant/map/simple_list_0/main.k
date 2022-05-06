data0 = map e in [{"name": "1", "value": 1}, {"name":"2", "value": 2}] {{ "name": e.name, "value": int(e.value) ** 2 }}
data1 = map e in [{"name": "1", "value": 1}, {"name":"2", "value": 2}] {
    {"name": e.name, "value": int(e.value) ** 2}
}
_var = [{"name": "1", "value": 1}, {"name":"2", "value": 2}]
data2 = map e in _var {{ "name": e.name, "value": int(e.value) ** 2 }}
data3 = map e in _var {
    {"name": e.name, "value": int(e.value) ** 2}
}