schema Data:
    labels: {str:}

schema Config:
    data: Data

data = Config {
    data.labels.key: "value"
}

config = Config {
    **(data if data else {
        data.labels.key: "value"
    })
}
