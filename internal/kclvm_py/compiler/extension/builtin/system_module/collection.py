from copy import deepcopy

import kclvm.api.object.internal.common as common


def KMANGLED_union_all(data: list) -> dict:
    if not data:
        return {}
    data_copy = deepcopy(data)
    value = data[0]
    for d in data_copy[1:]:
        common.union(value, d)
    return value
