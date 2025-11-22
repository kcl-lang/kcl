schema ObjectMeta:
    name?: str = None
    tagged?: bool = None
    version?: int = None
    historyVersions?: [str] = None
    labels?: {str:str} = None
