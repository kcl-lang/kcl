schema Config:
    name: str
    args: [str]

schema Data:
    _config: Config {
        name = "config"
        args: ["kcl", "-d"]
        args = ["kcl"]
        name = "kcl"
    }
    config: Config = _config | {name = "override"}

data = Data {}
