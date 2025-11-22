schema Config:
    getGroup: bool = True
    group?: str
    if getGroup:
        _app = "app"
        _tenant = "tenant"
        group = _app + " " + _tenant

config = Config {}
