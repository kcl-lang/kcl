schema Son:
    metadata?: {str:}
    name?: str
    key?: str
               
son1 = Son {
    metadata.labels.key: 123, name: "Bob"
    key: "123"
}

son2 = Son {metadata.labels.key: 456, name: "Alice", "key": "123"}

son3 = Son {
    metadata: {}
    name: "Green"
    key: "key"
}

son4 = Son {
    metadata.key: "val"
    name: None
    key: None
}
