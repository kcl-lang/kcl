schema Config:
    spec: {str:} = {
        replicas: 1
        data: [0]
    }

data = Config {
    spec.labels: {key: "value"}
    spec: {replicas = 2}
    spec.data += [1]
}
