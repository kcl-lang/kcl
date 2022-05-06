# Copyright 2021 The KCL Authors. All rights reserved.

from typing import Dict

from .parser import Type, KeyType

PROTOBUF_SCALAR_TYPE_TO_KCL_MAPPING: Dict[str, str] = {
    "int32": "int",
    "int64": "int",
    "uint32": "int",
    "uint64": "int",
    "double": "float",
    "float": "float",
    "fixed32": "int",
    "fixed64": "int",
    "sfixed32": "int",
    "sfixed64": "int",
    "bool": "bool",
    "string": "str",
    "bytes": "str",
}

KCL_SCALAR_TYPE_TO_PROTOBUF_TYPE_MAPPING: Dict[str, Type] = {
    "int": Type.INT64,
    "float": Type.DOUBLE,
    "bool": Type.BOOL,
    "str": Type.STRING,
}

KCL_SCALAR_TYPE_TO_PROTOBUF_KEY_TYPE_MAPPING: Dict[str, Type] = {
    "int": KeyType.INT64,
    "float": KeyType.INT64,
    "bool": KeyType.BOOL,
    "str": KeyType.STRING,
}
