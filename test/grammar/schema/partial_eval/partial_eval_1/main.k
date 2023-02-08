schema Spec:
    id: int
    value: str

schema Config:
    name?: str
    spec: Spec {
        id = 1
    }

config = Config {
    spec.value = "value"
}
