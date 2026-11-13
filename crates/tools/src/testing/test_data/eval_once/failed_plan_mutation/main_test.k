# Each case mutates the global `counter` in place and fails only to plan
# the package, so neither must see the other's change, as when the
# package is evaluated for each case alone.
test_bump = lambda {
    assert bump(counter).n == 1, "n is {}".format(counter.n)
}

test_bump_again = lambda {
    assert bump(counter).n == 1, "n is {}".format(counter.n)
}
