schema Config:
    main: Main

schema Main:
    name: str
    env: [Env]

schema Env:
    name: str
    value: str

_main = Main {
    name: "main"
    env: [
        {name: "ENV_1", value: "1"}
    ]
}

config: Config {
    main: _main
}

config: Config {
    main.name: "main"
    main: Main {
        env += [
            Env {name: "ENV_2", value: "2"}
        ]
    }
}
