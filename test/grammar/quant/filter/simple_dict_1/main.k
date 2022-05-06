data0 = filter k, v in { "a": "foo", "b": "bar" } { k == "a" }
data1 = filter k, v in { "a": "foo", "b": "bar" } {
    k == "a" and v == "foo"
}
