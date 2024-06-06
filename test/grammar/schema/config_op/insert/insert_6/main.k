schema x:
    m: [str] = ["hello"]
    nn = m

s = x {
}

t = s | {
    m += ["world"]
} | {
    m = ["world"]
} | {
    m += ["hello"]
}