schema ConfigMapping:
    [str]: Config

schema Config:
    [str]: any

configMapping: ConfigMapping {
    key = Config {
        data.key: "value"
    }
}

configMapping: ConfigMapping {
    key = Config {
        data.key: "value1"
    }
}
