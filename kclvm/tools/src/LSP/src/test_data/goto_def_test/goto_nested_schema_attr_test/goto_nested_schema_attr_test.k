schema Foo:
    data: Data


schema Data:
    spec: Spec

schema Spec:
    config: Config

schema Config:
    template: Template

schema Template:
    name: str

foo = Foo {
    data: {
        spec: {
            config: {
                template: {
                    name: "template"
                }
            }
        }
    }
}