schema Env:
    name: str
    value: str

schema Main:
    env: [Env]

schema Config:
    main: Main

config = Config {
    main: Main {
        env: [
            {name: "ENV_1", value: "1"}
        ]
    }
    main: Main {
        env += [
            {name: "ENV_2", value: "2"}
        ]
        env += [
            {name: "ENV_3", value: "3"}
        ]
    }
}
