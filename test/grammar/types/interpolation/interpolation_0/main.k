schema App:
    name: str
    containers?: {str: Container}

schema Container:
    image: str

app: App {
    name = "ngnix"
    containers = {
        "${name}" = {image = name}
    }
}
