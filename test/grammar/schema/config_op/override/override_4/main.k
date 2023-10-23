schema MySubSchema:
    a: int
    b: str
    c: {str:str}
    d: str = c[b]

schema MyTestSchema:
    field1?: [MySubSchema] = [{a: 1}]

instance = MyTestSchema {field1 = None}
