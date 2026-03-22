# Test for issue #2074: augmented assignment followed by subscript assignment
# in if block caused the subscript assignment to be silently ignored.

_map = {}
if True:
    _map |= {a: 1}
    _map["b"] = 2
result = _map
