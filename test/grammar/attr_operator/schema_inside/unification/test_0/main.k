schema Config:
    name?: str
    args?: [str]

_config: Config {
    name = "config"
}

_config: Config {
    args: ["kcl"]
}

config = _config
