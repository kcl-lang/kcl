schema Config:
    name: str
    args: [str]

schema Data:
    config: Config = {
        name = "config"
        args: ["kcl", "-d"]
        args += ["-p"]
        args += ["-o", "stdout"]
    }

data = Data {}
