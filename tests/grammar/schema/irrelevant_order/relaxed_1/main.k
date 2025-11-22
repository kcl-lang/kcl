schema Fib:
    n?: int = None
    n1 = n - 1
    n2 = n1 - 1
    value?: int = None

    if n <= 1:
        value = 1
    elif n == 2:
        value = 1
    else:
        value = (Fib {n: n1}).value + (Fib {n: n2}).value

fib8 = (Fib {n: 8}).value
