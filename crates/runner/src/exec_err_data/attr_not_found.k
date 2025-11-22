schema Data:
    resource?: str | {str:str}

schema Config[inputData]:
    data: Data = inputData
    template: str = data?.resource?.template

c = Config({resource = "100Gi"})
