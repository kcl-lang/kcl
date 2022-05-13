schema Config:
    name?: str
    args?: [str]

config: Config {
    name = "config"
    args: ["kcl", "-d"]
}

config: Config {
    name = "config_new"
    args += ["-p"]
    args += ["-o", "out.txt"]
}
