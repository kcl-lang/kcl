schema Fib:
    n1: int = n - 1
    n2: int = n1 - 1
    n: int
    value: int

    if n <= 1:
        value = 1
    elif n == 2:
        value = 1
    else:
        value = (Fib {n: n1}).value + (Fib {n: n2}).value

fib8 = (Fib {n: 8}).value
