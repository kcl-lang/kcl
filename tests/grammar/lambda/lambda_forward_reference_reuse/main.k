# Test for lambda forward reference with multiple invocations
# This test ensures that when a lambda is called multiple times before being defined,
# each invocation should work correctly with different arguments

# Multiple calls before definition
result1 = double(5)
result2 = double(10)
result3 = double(20)

# Define the lambda
double = lambda x: int {
    x * 2
}

# More calls after definition
result4 = double(3)
result5 = double(7)

# Test with nested lambdas called multiple times
a = transform(100)
b = transform(200)

transform = lambda value: int {
    addOne(value)
}

addOne = lambda x: int {
    x + 1
}

c = transform(300)
d = addOne(50)
