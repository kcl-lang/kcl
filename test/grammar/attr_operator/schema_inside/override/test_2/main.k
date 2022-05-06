schema Config:
    name: str
    args: [str]

schema Data:
    config: Config {
        name = "config"
        args: ["kcl", "-d"]
        args = ["kclvm"]
        name = "kclvm"
    }

data = Data {}
