import runtime

schema Port:
    number: int

    check:
        number > 0, "port number must be positive"

msg = runtime.catch(lambda {
    Port {number = -1}
})
assert msg == "port number must be positive"
# The caught check failure must not leak into this error.
assert False, "own message"
