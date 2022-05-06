schema Config:
    name?: str
    age?: int
    check:
        age > 0 if age

_config = Config {
    name: "Alice"
    age: 18
}
_config.age = -1
