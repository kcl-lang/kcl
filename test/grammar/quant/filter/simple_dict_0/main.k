data0 = filter _, v in { "a": "foo", "b": "bar" } { v == "foo" }
data1 = filter _, v in { "a": "foo", "b": "bar" } {
    v == "foo"
}
_a = { "a": "foo", "b": "bar" }
data2 = filter _, v in _a { v == "foo" }
data3 = filter _, v in _a {
    v == "foo"
}