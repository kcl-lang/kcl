# Test 1: Basic accumulation with non-commutative reducer
test1 = reduce(lambda acc: int, item: int -> int { acc * 10 + item }, [1, 2, 3], 9)

# Test 2: Different accumulator/item types
test2 = reduce(lambda acc: [int], item: int -> [int] { acc + [item * 2] }, [1, 2, 3], [7])

# Test 3: Int to float coercion
test3 = reduce(lambda acc: float, item: float -> float { acc / 10.0 + item }, [1, 2, 3], 5)

# Test 4: Empty list with initial: returns initial
test4 = reduce(lambda acc: int, item: int -> int { assert False, "should not be called" }, [], 99)

# Test 5: Without initial: first element as seed
# Uses primes-1: [1, 3-1, 5-1] = [1, 2, 4]; reducer adds 1 back: 1 * 3 * 5 = 15
test5 = reduce(lambda acc: int, item: int -> int { acc * (item + 1) }, [1, 2, 4])

# Test 6: 1-element without initial: returns first element
test6 = reduce(lambda acc: int, item: int -> int { assert False, "should not be called" }, [42])

# Test 7: Empty list without initial and nullable acc: returns None
test7 = reduce(lambda acc: any, item: any -> any { assert False, "should not be called" }, [])
