schema Config:
    name?: str
    age?: int
    hc: [int]

config = Config {
    name: "Alice"
}

config = Config {
    age: 18
}

config = Config {
    hc: [1]
}

data = [1, 2, 3]
