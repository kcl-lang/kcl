schema Env:
    name: str
    value: str

schema Main:
    env: [Env]

schema Config:
    main: Main

_main = Main {
    env: [
        {name: "ENV_1", value: "1"}
    ]
}

config = Config {
    main: _main
    main: Main {
        env = [
            {name: "ENV_2", value: "2"}
        ]
    }
}
