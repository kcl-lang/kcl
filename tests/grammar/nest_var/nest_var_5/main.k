schema Metadata:
    name: str
    labels: {str:}

schema Person:
    metadata: Metadata

person = Person {
    metadata.name: "person"
    metadata.labels.key1: "value1"
    metadata.labels.key2: "value2"
}
