schema Config:
    getGroup: bool = False
    group?: str
    if getGroup:
        _app = "app1"
        _tenant = "tenant1"
    else:
        _app = "app2"
        _tenant = "tenant2"
    group = _app + " " + _tenant        

config = Config {}
