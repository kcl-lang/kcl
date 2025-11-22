schema Config:
    name: str
    args: [str]

schema Data:
    config: Config = Config {
        name = "config"
        args: ["kcl", "-d"]
        args += ["-p"]
    }

data = Data {}
