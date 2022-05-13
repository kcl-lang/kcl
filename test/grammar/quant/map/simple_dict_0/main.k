data0 = map _, v in { "a": "foo", "b": "bar" } { v }
data1 = map _, v in { "a": "foo", "b": "bar" } {
    v
}
_a = { "a": "foo", "b": "bar" }
data2 = map _, v in _a { v }
data3 = map _, v in _a {
    v
}
