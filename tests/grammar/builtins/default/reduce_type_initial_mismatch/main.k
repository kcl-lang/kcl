# Initial type mismatch: string initial passed to int accumulator
result = reduce(lambda acc: int, item: int -> int { acc + item }, [1], "not an int")
