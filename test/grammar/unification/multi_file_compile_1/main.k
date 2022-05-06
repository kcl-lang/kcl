schema Config:
    name?: str
    args: [str]
    labels: {str:}

config = Config {
    name: "config1"
    args: ["kcl", "main.k"]
    labels.key1: "value1"
}
