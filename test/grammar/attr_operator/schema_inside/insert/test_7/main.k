schema Config:
    name: str
    args: [str] = ["kcl", "-d"]

schema Data:
    config: Config = {
        name = "config"
        args += ["-p"]
        args += ["-o", "stdout"]
    }

data = Data {}
