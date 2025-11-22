schema Spec:
    id: int
    value: str

schema Config:
    spec: Spec = Spec {
        id = 1
    }

schema App:
    config: Config

app = App {
    config.spec.value = "value"
}
