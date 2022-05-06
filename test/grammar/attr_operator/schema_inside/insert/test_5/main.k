schema Config:
    name: str
    args: [str]

schema Data:
    _config: Config {
        name = "config"
        args: ["kcl", "-d"]
        args += ["-p"]
        args += ["-o", "stdout"]
    }
    config: Config = _config

data = Data {}
