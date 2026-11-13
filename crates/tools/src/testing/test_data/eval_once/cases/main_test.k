import runtime

test_pass = lambda {
    print("pass")
    assert double(2) == 4
}

test_fail = lambda {
    print("fail")
    assert double(2) == 5, "double(2) is not 5"
}

test_pass_after_fail = lambda {
    print("pass after fail")
    assert len(Item.instances()) == 3
}

test_option = lambda {
    assert option("who") == "kcl", "got {}".format(option("who"))
}

test_new_instance = lambda {
    extra = Item {n = 9}
    assert len(Item.instances()) == 4
}

test_new_instance_again = lambda {
    extra = Item {n = 10}
    assert len(Item.instances()) == 4
}

test_runtime_error = lambda {
    print("runtime error")
    [1][3]
}

test_catch = lambda {
    print("catch")
    msg = runtime.catch(lambda {
        assert False, "caught"
    })
    assert "caught" in msg
}

test_pass_after_catch = lambda {
    print("pass after catch")
    assert items[2].n == 2
}
