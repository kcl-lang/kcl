schema Config:
    name: str
    env?: str

env = "env"

data1 = Config {
    if env == "env": name: env
    else: name: "name"
}
data2 = Config {
    if env != "env": name: env
    else: name: "name"
}
