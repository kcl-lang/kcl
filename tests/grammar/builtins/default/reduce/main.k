# Test 1: Basic invocation - proves reducer is called (acc + 1 ignores item)
test1 = reduce(lambda acc: int, item: int -> int { acc + 1 }, [0], 5)

# Test 2: Int to float coercion - int initial coerced to float
test2 = reduce(lambda acc: float, item: float -> float { acc + 0.5 }, [0], 1)
