schema Test:
    app: str = ""
    name: str
    meta: {str:} = {
        "name": "value",
        "test": app
    }
    labels: {str:} = {
        "otherLabels": name,
        **meta
    }

instance = Test {
    app = "override_app"
    name = "value"
}
