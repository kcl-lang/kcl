import regex
data = {
    k1 = "1"
}
annotations = {
    k1 = "1"
    k2 = "2"
    k3 = "3"
}
result = all k, v in annotations {
    regex.match(v, data[k]) if data[k]
}
