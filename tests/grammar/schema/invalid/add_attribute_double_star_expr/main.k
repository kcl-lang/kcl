innerConfig = {
    nam = ""
    config = {
        nam = ""
    }
}

config = {
    **innerConfig
}

schema Config:
    name: str
    config: ConfigInner

schema ConfigInner:
    name: str

c = Config {
    **config
}
