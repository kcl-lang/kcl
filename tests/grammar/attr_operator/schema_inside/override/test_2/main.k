schema Config:
    name: str
    args: [str]

schema Data:
    config: Config {
        name = "config"
        args: ["kcl", "-d"]
        args = ["kcl"]
        name = "kcl"
    }

data = Data {}
