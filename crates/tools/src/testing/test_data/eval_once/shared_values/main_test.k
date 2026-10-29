# Each case mutates the global `counter` in place through the lambda
# parameter `c`. When the package is evaluated once for all the cases,
# the second case sees the first one's change.
test_first_bump = lambda {
    # Code that returns normally leaves the evaluation as it was, so the
    # second case still runs in it.
    squares = {"k${i}": i * i for i in range(4) if i > 0}
    extra = Counter {n = squares.k3}
    config = {
        name = "c"
        if extra.n > 4:
            big = True
    }
    assert config.big
    assert all _, v in squares {
        v > 0
    }
    assert sorted([extra.n, 1]) == [1, 9]
    assert typeof(extra) == "Counter"
    assert bump(counter).n == 1, "n is {}".format(counter.n)
}

test_second_bump = lambda {
    assert bump(counter).n == 1, "n is {}".format(counter.n)
}
