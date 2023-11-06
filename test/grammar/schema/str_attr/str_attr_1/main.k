schema Data:
    "spec": Spec

schema Spec:
    "config": Config

schema Config:
    "template": Template

schema Template:
    "name": str

data = Data {
    spec: {
        config: {
            template: {
                name: "template"
            }
        }
    }
}
name = data["spec"]["config"]["template"].name
