schema Data:
    [name: str]: str
    check:
        name in ["Alice", "Bob", "John"]

data = Data {
    key1: "value1"
}
