appConfiguration: AppConfiguration {
    atsValues = values._v
    resource: res.Resource {
        cpu = "2"
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