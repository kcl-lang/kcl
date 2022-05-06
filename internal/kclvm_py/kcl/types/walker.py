# Copyright 2020 The KCL Authors. All rights reserved.

from typing import Callable, cast

import kclvm.api.object as objpkg

from .type import Type


def WalkType(tpe: Type, walk_fn: Callable):
    """Walk one type recursively and deal the type using the `walk_fn`"""
    if not tpe or not isinstance(tpe, Type):
        return tpe
    tpe = walk_fn(tpe)
    if tpe.type_kind() == objpkg.KCLTypeKind.UnionKind:
        tpe = cast(objpkg.KCLUnionTypeObject, tpe)
        types = []
        for t in tpe.types:
            t = WalkType(t, walk_fn)
            if t:
                types.append(t)
        tpe.types = types
    elif tpe.type_kind() == objpkg.KCLTypeKind.ListKind:
        tpe = cast(objpkg.KCLListTypeObject, tpe)
        tpe.item_type = WalkType(tpe.item_type, walk_fn)
    elif tpe.type_kind() == objpkg.KCLTypeKind.DictKind:
        tpe = cast(objpkg.KCLDictTypeObject, tpe)
        tpe.key_type = WalkType(tpe.key_type, walk_fn)
        tpe.value_type = WalkType(tpe.value_type, walk_fn)
    return tpe
