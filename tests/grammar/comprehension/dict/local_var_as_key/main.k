_data = {
    "a": 'foo'
    "b": 'bar'
}

r0 = [{v = k} for k, v in _data]
r1 = [{k = v} for k, v in _data]
r2 = [{k.foo = v} for k, v in _data]
r3 = [[k] for k, v in _data]
r4 = [[k, v] for k, v in _data]
