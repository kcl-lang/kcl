appConfiguration: AppConfiguration {
    releaseStrategy: "Manual"
    replicas: 3
    mainContainer: container.Main {
        env = [
            {
                name: "key1"
                value: "v1"
            }
            {
                name: "key2"
                value: "v2"
            }
        ]
    }
    resource: res.Resource {
        disk = "35Gi"
    }
}