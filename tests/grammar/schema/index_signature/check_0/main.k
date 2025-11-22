schema Data:
    [name: str]: str
    check:
        name in ["Alice", "Bob", "John"]

lowerMap = Data {
    Alice: "alice"
    Bob: "bob"
    John: "john"
}
