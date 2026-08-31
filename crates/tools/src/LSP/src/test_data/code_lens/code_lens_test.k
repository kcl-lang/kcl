func = lambda x: str -> str {
    x
}

test_func_0 = lambda {
    assert func("a") == "a"
}

not_a_test = lambda {
    assert True
}

test_func_1 = lambda {
    assert func("b") == "b"
}

a = 1
