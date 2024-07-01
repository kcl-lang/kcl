import runtime

msg = runtime.catch(lambda {
    assert False, "error msg"
})
