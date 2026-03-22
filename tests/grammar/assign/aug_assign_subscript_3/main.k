# Test for issue #2074: |= on list followed by subscript assignment in if block
_list = []
if True:
    _list |= [1]
    _list[0] = 99
result = _list
