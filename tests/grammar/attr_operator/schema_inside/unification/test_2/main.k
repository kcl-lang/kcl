schema Config:
    name: str
    labels: {str:}

schema Data:
    config: Config {
        name = "config"
        labels.key1: "value1"
        labels.key2: "value2"
    }

data = Data {}
