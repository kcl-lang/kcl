schema Config:
    id: int
    name?: str

configOrigin = Config {
    id = 1
    name = "origin config name"
}
configNew = configOrigin {
    name = "override config name"
}
