# Test that reducer errors propagate with context
result = reduce(lambda acc: int, item: int -> int { assert False, "reducer error" }, [1], 5)
