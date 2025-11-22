schema User:
    name: str
    age: int
    message?: str
    data: Data
    labels: {str:}
    hc: [int]

schema Data:
    id: int
    value: str
