# The case mutates the global `a` in place through the lambda parameter
# `xs`. Planning the package after the case then finds `a` conflicting
# with the config expression; planning it before the case does not.
test_change = lambda {
    change(a)
    assert a == [2]
}
