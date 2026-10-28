# The first case mutates the global `counter` in place and then fails, so
# the second case must see `counter` as the package defines it, as it does
# when the package is evaluated for it alone.
test_bump_then_fail = lambda {
    assert bump(counter).n == 1, "n is {}".format(counter.n)
    assert False, "fails after the bump"
}

test_counter_is_fresh = lambda {
    assert counter.n == 0, "n is {}".format(counter.n)
}
