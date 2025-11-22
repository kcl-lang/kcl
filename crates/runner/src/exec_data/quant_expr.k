x0 = map i in [1, 2, 3] {
    i + 1 if i > 1
}
x1 = filter i in [1, 2, 3] {
    i == 1 if i
}
x2 = all i in [1, 2, 3] {
    i > 1
}
x3 = any i in [1, 2, 3] {
    i > 1
}
x4 = filter i, e in [{"name": "1", "value": 1}, {"name":"2", "value": 2}] {
    int(e.value) > 1 and i > 0
}
x5 = filter i, e in [{"name": "1", "value": 1}, {"name":"2", "value": 2}] {
    int(e.value) >= 1 and i == 0
}
x6 = filter k, v in { "a": "foo", "b": "bar" } {
    k == "a"
}
x7 = filter k, v in { "a": "foo", "b": "bar" } {
    k == "a" and v == "foo"
}
