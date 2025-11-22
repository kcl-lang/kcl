schema Config:
    name?: str

makeCopy = lambda p: Config -> Config {
    Config {name = p.name + "-copy"}
}
configs = {
    "foo": Config {name = "foo"}
    "bar": Config {name = "bar"}
}
copies = {"${name}-copy": makeCopy(config) for name, config in configs}
