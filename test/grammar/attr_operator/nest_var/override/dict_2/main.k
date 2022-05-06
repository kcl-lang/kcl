schema Config:
    spec: {str:} = {
        replicas: 1
    }

data = Config {
    spec.labels: {key: "value"}
    spec: {replicas = 2}
}
