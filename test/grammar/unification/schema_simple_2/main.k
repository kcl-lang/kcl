schema Config:
    name?: str
    age?: int
    hc: [int]

config = Config {
    name: "Alice"
}
age = 18
config = Config {
    age: age
}

config = Config {
    hc: [1]
}
