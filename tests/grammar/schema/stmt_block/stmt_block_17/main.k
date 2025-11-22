schema Test:
    app: str
    meta: {str:str} = {
        "name": "value",
        "test": app
    }
    labels: {str:str} = {
        "otherLabels": "value",
        **meta
    }

instance = Test {
    app: "override_app"
}
