data0 = filter i, e in [{"name": "1", "value": 1}, {"name":"2", "value": 2}] { int(e.value) > 1 and i > 0 }
data1 = filter i, e in [{"name": "1", "value": 1}, {"name":"2", "value": 2}] {
    int(e.value) >= 1 and i == 0
}
