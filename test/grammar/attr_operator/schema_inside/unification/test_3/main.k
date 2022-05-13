schema Config:
    name: str
    labels: {str:}

schema Data:
    _config: Config {
        name = "config"
        labels.key1: "value1"
        labels.key2: "value2"
    }
    config: Config = _config | {name = "override"}

data = Data {}
