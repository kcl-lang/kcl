schema Person:
    name: str = "Alice"

    check:
        all c in name {
            c.isupper()
        }

data0 = all i, c in "app" {
    c.upper() if 1 > 1
}
data1: bool = all i in [1, 2, 3] {
    i if i > 2
}
data2: bool = any i in [1, 2, 3] {
    i if i > 2
}
data3: [str] = map k, v in {key1: "value1", key2: "value2"} {
    k + v
}
data4: [int] = filter i, elem in [1, 2, 3] {
    i > 1 and elem > 2
}
value = [0] or ["0"] 
data5: bool = all v in value {
    int(v) > 1
}
