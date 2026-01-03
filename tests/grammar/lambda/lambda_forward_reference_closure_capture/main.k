# Test for lambda forward reference with closure capture
# This test ensures that closures are properly captured even when
# functions are called before being defined

# Call multiplier before it's defined
result1 = multiplier(5)

# Define multiplier that captures factor from outer scope
factor = 10
multiplier = lambda x: int {
    x * factor
}

# Test closure works correctly
result2 = multiplier(3)

# Test with nested closure capture
a = applyOp(10, 20)

applyOp = lambda x: int, y: int {
    baseOp(x, y)
}

baseOp = lambda a: int, b: int {
    a + b
}

b = applyOp(30, 40)
c = baseOp(5, 15)
