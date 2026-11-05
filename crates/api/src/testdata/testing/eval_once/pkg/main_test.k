# Each case mutates the global `counter` in place through the lambda
# parameter `c`. When the package is evaluated once for all the cases,
# the second case sees the first one's change.
test_first_bump = lambda {
    assert bump(counter).n == 1, "n is {}".format(counter.n)
}

test_second_bump = lambda {
    assert bump(counter).n == 1, "n is {}".format(counter.n)
}
