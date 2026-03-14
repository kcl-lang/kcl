# Test that reducer error mid-iteration propagates
result = reduce(lambda acc: int, item: int -> int { assert item != 2, "error at item 2" }, [1, 2, 3], 0)
