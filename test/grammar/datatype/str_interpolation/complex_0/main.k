schema Data:
    a: int = 1
    b: float = 1.2
    c: bool = True
    d: [int] = [1]
    e: {str:str} = {"key": "value"}
    data: str = "${a} ${b} ${c} ${d} ${e}"

data = Data {}
