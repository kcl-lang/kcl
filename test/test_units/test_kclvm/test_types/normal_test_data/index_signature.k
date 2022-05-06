schema Data:
    [name: str]: str

    check:
        name.islower()

data = Data {
    key: "value"
}
value = data.key
