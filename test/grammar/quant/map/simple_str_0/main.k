data0 = map e in "abC" { e.islower() }
data1 = map e in "abC" {
    e.isupper()
}
_var = "abC"
data2 = map e in _var { e.islower() }
data3 = map e in _var {
    e.isupper()
}
