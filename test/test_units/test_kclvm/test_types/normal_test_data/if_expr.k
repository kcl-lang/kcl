a = 1 if True else "ss"
b = "1" if a else 1
c = 1 + 2 if True else 3
d: int|str = 1
e: bool = d == "s" 
f: [int] = [1, 2] + ([3, 4] + [5, 6] if True else [7, 8])
