schema Config:
    name: str
    env: str

env = "env"

data1 = Config {
    if env == "env":
        name: env
        env: env
    else:
        name: "name"
        env: "name"
}
data2 = Config {
    if env != "env":
        name: env
        env: env
    else:
        name: "name"
        env: "name"
}
