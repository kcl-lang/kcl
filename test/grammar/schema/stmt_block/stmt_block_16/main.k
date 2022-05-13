_dictlist = [{"key1": "value1", "key2": "value2"}, {"key1": "value3", "key2": "value4"}]
_keys = ["key1", "key2"]

schema Person:
    names: {str:} = {dl[k]: k + ":" + dl[k] for dl in _dictlist for k in _keys}

alice = Person {}
