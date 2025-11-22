schema Config:
    name: str = "defualt"
    help: str = name
    labels?: {str:} = None
    metaLabels?: {str:} = labels
    overQuota: bool = False

schema AppConfig(Config):
    appName: str = name
    name: str = "app"

    if overQuota:
        labels = {
            "key1": "value1"
            "key2": "value2"
        }

config = Config {
    name = "override", 
    overQuota = True
}

appConfig = AppConfig {
    overQuota = True
}
