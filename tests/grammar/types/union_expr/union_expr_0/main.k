schema Container:
    name: str
    image: str
    volumeMounts: [{str:}]

config = {
    image = "test/test-container:test-cluster"
    volumeMounts = [{
        name = "config"
        mountPath = "/app/config"
    }]
}

expected: Container = config | {name = "test-repo/test-image:test-tag"}
