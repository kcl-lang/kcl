schema Data:
    id: int = 1

schema Config[data: Data]:
    spec: {str:} = {
        internal.data = data.id
        id = data.id
    }

Func = lambda data: Data {
    {
        internal.data = data.id
        id = data.id
    }
}
spec = Func(Data())
config = Config(Data())
