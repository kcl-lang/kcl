schema Config:
    getGroup: bool = True
    group?: str
    if getGroup:
        _app = "app"
        _tenant = _app.upper() if _app else "tenant"
        group = _app + " " + _tenant

config = Config {}
