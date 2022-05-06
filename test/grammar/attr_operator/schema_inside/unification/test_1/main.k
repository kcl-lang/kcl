schema Config:
    name?: str
    args?: [str]
    id?: int

_config: Config {
    name = "config"
}

_config: Config {
    args: ["kcl"]
}

_config: Config {
    id: 1
}

config = _config
