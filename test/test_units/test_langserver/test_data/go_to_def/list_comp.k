a = e = 1

schema S:
    a: str
    b: [str]
    c: [str] = [a for a in b]
    d: int = e
