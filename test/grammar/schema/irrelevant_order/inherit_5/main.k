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
        }
    if overQuota:
        labels = {
            "key2": "value2"
        }

config = Config {
    overQuota = False
}

appConfig = AppConfig {
    appName = "myApp"
    overQuota = True
}
