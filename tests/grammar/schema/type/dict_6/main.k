schema Data:
    labels: {str:}

schema Config:
    data: Data

data = Data {
    labels.key: "value"
}

config = Config {
    data: data if data else {
        labels.key: "value"
    }
}
