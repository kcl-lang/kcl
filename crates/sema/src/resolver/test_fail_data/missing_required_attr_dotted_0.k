# Case B: dotted-path keys. The pre-processor desugars `main.name` into
# `main: {name: "main"}`, so the two siblings `main: {name = "main"}` and
# `main: Main {...}` together provide both `name` and `env`. The static
# check should NOT report `name` as missing.
schema Main:
    name: str
    env: [str]

schema Config:
    main: Main

config: Config {
    main.name: "main"
    main: Main {
        env: []
    }
}