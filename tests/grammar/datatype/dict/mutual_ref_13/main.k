schema Data:
    id: int = 1

schema Config[data: Data]:
    spec: {str:} = {
        internal.data = data.id
        id = data.id
    }

Func = lambda data: Data {
    {
        # Note data here is a local schema variable
        internal: {"data" = data.id}
        id = data.id
    }
}
spec = Func(Data())
config = Config(Data())
