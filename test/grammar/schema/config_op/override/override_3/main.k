schema MySubSchema:
    a: int
    b: str
    c: {str:str}
    d: str = c[b]

schema MyTestSchema:
    field1?: [MySubSchema] = [{a = 1, b = "key", c = {key = "value"}}]

instance = MyTestSchema {field1: None}
