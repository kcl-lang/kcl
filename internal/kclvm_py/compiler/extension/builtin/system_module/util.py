from typing import Any

from kclvm.api.object import KCLNumberMultiplierObject
from kclvm.api.object.internal import Undefined, UndefinedType

import kclvm.kcl.info as kcl_info


def filter_fields(
    value: Any, ignore_private: bool = False, ignore_none: bool = False
) -> Any:
    """Remove private attributes start with '_' and None value in data"""
    if not value:
        return value
    if value is Undefined or isinstance(value, UndefinedType):
        return value
    if isinstance(value, KCLNumberMultiplierObject):
        return str(value)
    if isinstance(value, list):
        return [
            filter_fields(_v, ignore_private, ignore_none)
            for _v in value
            if ignore_none and _v is not None or not ignore_none
            if not isinstance(_v, UndefinedType)
        ]
    elif isinstance(value, dict):
        return {
            filter_fields(_k, ignore_private, ignore_none): filter_fields(
                _v, ignore_private, ignore_none
            )
            for _k, _v in value.items()
            if not kcl_info.isprivate_field(_k) and ignore_private or not ignore_private
            if ignore_none and _v is not None or not ignore_none
            if not isinstance(_v, UndefinedType)
        }
    elif isinstance(value, (int, float, str, bool)):
        return value
    else:
        raise Exception("Invalid KCL Object")
