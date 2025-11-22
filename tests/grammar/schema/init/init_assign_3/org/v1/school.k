import ...metadata.v1

schema School:
    kind?: str = "School"
    metadata?: v1.ObjectMeta
    address?: str = None
    famous?: bool = None
    subjects?: [Subject] = None
    partitions?: [str] = None
    principals?: {str:str} = None
