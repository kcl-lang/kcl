schema Resource:
    cpu: int
    memory: str

schema Config:
    resource: Resource

r = Resource {
    cpu = 4
    memory = "8Gi"
}

config: Config {
    resource: Resource {
        cpu = 2
        memory = "4Gi"
    }
}

config: Config {
    resource: r | {
        cpu = 8
        memory = "16Gi"
    }
}
