schema Person:
    version: str

schema Son(Person):
    metadata: {str:} | str
    name: str
               
son = Son {
    version: "v1"
    name: "Alice"
    metadata.labels.key: 123
}
