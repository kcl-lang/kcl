test_sum_positive = lambda {
    assert sum(2, 3) == 5
}

test_sum_zero = lambda {
    assert sum(0, 0) == 0
}

test_negate = lambda {
    assert negate(7) == -7
}

# Only the "a" branch of `grade` is exercised. Tests for "b" and "c"
# intentionally do not exist so coverage can flag the corresponding
# lines in lib.k as uncovered.
test_grade_a = lambda {
    assert grade("a") == "ok:1"
}
