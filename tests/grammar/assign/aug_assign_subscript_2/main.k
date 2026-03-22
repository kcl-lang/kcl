# Test for issue #2074: multiple subscript assignments after |= in if block
_map = {}
if True:
    _map |= {a: 1}
    _map["b"] = 2
    _map["c"] = 3
result = _map
