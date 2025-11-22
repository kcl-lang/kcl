schema Data:
    name: str
    metadata: {str:}
    spec: {str:}

data = Data {
    name = "release"
    metadata.name = name
    spec.template.metadata.name = metadata.name
}
