schema App:
    name: str
    containers?: {str: Container}

schema Container:
    image: str
    envs: {str:Env}

schema Env:
    value?: str
    valueFrom?: str

app: App {
    name = "ngnix"
    containers = {
        "${name}" = {
            image = name
            envs = {
                "${name}" = {value = "value"}
            }
        }
    }
}
