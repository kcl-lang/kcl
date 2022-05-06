schema Main:
    env: [str]
    args: [str]

schema Config:
    name?: str
    age?: int
    hc?: [int]
    main: Main

_main = Main {
    env: ["123", "456"]
    args: ["1"]
}

config = Config {
    name: "Alice"
    main: _main
}

config = Config {
    age: 18
    main: Main {
        env = ["789", "456"]
        args = ["1"]
    }
}
