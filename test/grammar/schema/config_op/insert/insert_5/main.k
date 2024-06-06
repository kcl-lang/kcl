schema x:
    m: [str] = ["hello"]

s = x {
}

t = s | {
    m += ["world"]
}
