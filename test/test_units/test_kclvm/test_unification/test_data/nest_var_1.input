schema Name0:
    name?: str
    data?: int

schema Name:
    name?: Name0

schema Config:
    args?: [str]
    labels: {str:}
    name: Name

config = Config {
    args: ["kcl", "main.k"]
    labels.key1: "value1"
    name.name.name: "name"
}

config = Config {
    labels: {
        key2: "value2"
    }
    name.name: Name0 {data: 1}
}
