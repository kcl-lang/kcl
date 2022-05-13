data0 = filter e in [{"name": "1", "value": 1}, {"name":"2", "value": 2}] { int(e.value) > 1 }
data1 = filter e in [{"name": "1", "value": 1}, {"name":"2", "value": 2}] {
    int(e.value) > 1
}
_var = [{"name": "1", "value": 1}, {"name":"2", "value": 2}]
data2 = filter e in _var { int(e.value) > 1 }
data3 = filter e in _var {
    int(e.value) > 1
}