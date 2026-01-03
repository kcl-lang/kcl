# Test for lambda forward reference with nested calls
# This test ensures that when lambda functions are called before they are defined,
# and one lambda calls another, each invocation should use the correct arguments
# and return the correct results (issue #1919)

# Call wrappedIdentity before it's defined
a = wrappedIdentity(1)

# Call identity before it's defined
b = identity(2)

# Define wrappedIdentity that calls identity
wrappedIdentity = lambda item: int {
    identity(item)
}

# Define identity
identity = lambda item: int {
    item
}

# Test calling both functions again after they are defined
c = wrappedIdentity(3)
d = identity(4)

# Test with more complex nesting
addOne = lambda x: int {
    x + 1
}

# Call before definition
e = add(10, 20)

add = lambda a: int, b: int {
    a + b
}

# Test multiple parameters
f = add(5, 10)
