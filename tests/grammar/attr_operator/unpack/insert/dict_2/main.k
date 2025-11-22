schema Config:
    val: [int] = [0]

base = {
    val = [2]
}

config = Config {
    **base
    val += [1]
}
