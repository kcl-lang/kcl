import runtime

schema Port:
    number: int

msg = runtime.catch(lambda {
    Port {number = None}
})
# The caught required-attribute error must not leak into this error.
a = [][0]
