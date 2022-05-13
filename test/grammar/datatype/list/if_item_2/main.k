schema Template:
    appName: str = "app"
    idc: str = ""
    result?: {str:} = {
        "env": [
            if appName: {
                "name": "APPNAME"
                "value": appName
            },
            if idc: {
                "name": "IDCNAME"
                "value": idc
            },
        ]
    }

template = Template {}
