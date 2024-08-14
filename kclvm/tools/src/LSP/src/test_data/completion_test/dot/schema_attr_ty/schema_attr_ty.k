schema A:
    n: str

schema B:
    named: A
    list: [A]
    dict: {str:A}
    union: str|A

a: A