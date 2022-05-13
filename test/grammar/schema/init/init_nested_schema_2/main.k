schema Config:
    replicas: int = 3

schema Data:
    config: Config = Config {
        replicas = 1
    }
    config.replicas = 4
    replicas: int = config.replicas

data = Data {}
_config = Config {
    replicas = 1
}
_config.replicas = 4
config = _config
replicas = _config.replicas
