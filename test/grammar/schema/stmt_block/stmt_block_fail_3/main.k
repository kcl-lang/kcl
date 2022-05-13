schema Parent:
    a?: str
    b?: str
    c?: str

schema Son:
    assert False, "assert in schema"

parent = Parent {}
son = Son {}
