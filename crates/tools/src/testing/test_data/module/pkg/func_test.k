test_func_0 = lambda {
    assert func("a") == "a"
}

test_func_1 = lambda {
    assert func("a") == "d"
}

test_func_2 = lambda {
    assert func("a") == option("a"), "got {}".format(option("a"))
}
