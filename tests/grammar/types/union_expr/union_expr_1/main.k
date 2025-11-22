schema Container:
    name: str
    image: str
    volumeMounts: [{str:}]

schema Config:
    _config = {
        image = "test/test-container:test-cluster"
        volumeMounts = [{
            name = "config"
            mountPath = "/app/config"
        }]
    }
    expected: Container = _config | {name = "test-repo/test-image:test-tag"}

config = Config {}
