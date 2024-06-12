appConfiguration : AppConfiguration {
    atsValues = values._v
    resource: res.Resource {
        cpu : {
            request: "100m"
            limit_plus: "20000m"
        }
        memory = "4Gi"
    }
    mainContainer: container.Main {env = [
        {
            name: "key1"
            value: "v1"
        }
        {
            name: "key2"
            value: "v2"
        }
    ]}
    replicas = 3
}