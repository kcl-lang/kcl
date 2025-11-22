schema Config:
    id: int
    values: [int]

configOrigin = Config {
    id = 1
    values = [0, 1]
}
configNew = configOrigin {
    id = 2
    values += [2, 3]
}
