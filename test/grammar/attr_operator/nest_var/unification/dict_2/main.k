schema Config:
    spec: {str:} = {
        replicas: 1
        env: [{"key1": "value1"}]
    }

data = Config {
    spec.labels: {key: "value"}
    spec: {replicas = 2}
    spec.env: [{"key2": "value2"}]
}
