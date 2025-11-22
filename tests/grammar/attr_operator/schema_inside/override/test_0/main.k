schema Config:
    name?: str
    args?: [str]

config: Config {
    name = "config"
    args: ["kcl", "-d"]
}

config: Config {
    args = ["kcl"]
}
