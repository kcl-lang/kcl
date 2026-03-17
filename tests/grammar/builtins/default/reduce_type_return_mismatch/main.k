# Return type mismatch: str return doesn't match int accumulator on second iteration
result = reduce(lambda acc: int, item: int -> str { str(acc) }, [1, 2], 0)
