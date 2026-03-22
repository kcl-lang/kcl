# Test for issue #2074: binary | reassignment followed by subscript assignment in if block
_map = {}
if True:
    _map = _map | {a: 1}
    _map["b"] = 2
result = _map
