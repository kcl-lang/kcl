schema Data:
    spec: Spec

schema Spec:
    config: Config

schema Config:
    template: Template

schema Template:
    [...str]: int
    name: str

data = Data {
    spec: {
        config: {
            template: {
                name: "template"
                id: 1
            }
        }
    }
}
