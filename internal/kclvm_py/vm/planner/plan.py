import io
import json
import inspect

from dataclasses import dataclass
from collections import OrderedDict
from typing import Dict, List
from ruamel.yaml import YAML

import kclvm
import kclvm.config
import kclvm.api.object as obj

from kclvm.api.object.internal import Undefined, UndefinedType
from kclvm.api.object.schema import (
    SETTINGS_OUTPUT_KEY,
    SCHEMA_SETTINGS_ATTR_NAME,
)


KCL_PLAN_TYPE = [
    obj.KCLObjectType.INTEGER,  # int
    obj.KCLObjectType.FLOAT,  # float
    obj.KCLObjectType.STRING,  # string
    obj.KCLObjectType.BOOLEAN,  # True, False
    obj.KCLObjectType.NUMBER_MULTIPLIER,  # 1M, 1Ki
    obj.KCLObjectType.NONE,  # None
    obj.KCLObjectType.UNDEFINED,  # Undefined
    obj.KCLObjectType.DICT,  # dict
    obj.KCLObjectType.LIST,  # list
    obj.KCLObjectType.SCHEMA,  # dict with __settings__
]
LIST_DICT_TEMP_KEY = "$"


def is_kcl_schema(value: dict):
    return (
        value is not None
        and isinstance(value, dict)
        and SCHEMA_SETTINGS_ATTR_NAME in value
    )


def order_dict(d: any) -> dict:
    result = {}
    for k, v in sorted(d.items()):
        if isinstance(v, dict):
            result[k] = order_dict(v)
        else:
            result[k] = v
    return result


def handle_schema(value: dict):
    # on kcl schema
    filtered = filter_results(value)
    if filtered is None:
        return filtered, False
    settings = SCHEMA_SETTINGS_ATTR_NAME
    output_type = SETTINGS_OUTPUT_KEY
    if settings in value and value[settings][output_type] == obj.SETTINGS_OUTPUT_IGNORE:
        if len(filtered) <= 1:
            return None, False
        else:
            return filtered[1:], True
    standalone = False
    if (
        settings in value
        and value[settings][output_type] == obj.SETTINGS_OUTPUT_STANDALONE
    ):
        standalone = True
    return filtered, standalone


def filter_results(keyvalues: dict) -> List[dict]:
    if keyvalues is None:
        return None

    # index 0 for in-line keyvalues output, index 1: for standalone keyvalues outputs
    results = [OrderedDict()]

    for key, value in keyvalues.items():
        if value is None and kclvm.config.disable_none:
            continue
        if isinstance(key, str) and key.startswith("_"):
            pass
        elif value is Undefined or isinstance(value, UndefinedType):
            pass
        elif inspect.isclass(value):
            pass
        elif inspect.isfunction(value):
            pass
        elif inspect.ismodule(value):
            pass
        elif inspect.isfunction(value):
            pass
        elif is_kcl_schema(value):
            filtered, standalone = handle_schema(value)
            if filtered is not None:
                if standalone:
                    # if the instance is marked as 'STANDALONE', treat it as a separate one and
                    # extend it and derived STANDALONE instances to results.
                    results.extend(filtered)
                else:
                    # else put it as the value of the key of results
                    if len(results) > 0 and len(filtered) > 0:
                        results[0][key] = filtered[0]
                    if len(filtered) > 1:
                        # if the value has derived 'STANDALONE' instances, extend them
                        results.extend(filtered[1:])
        elif isinstance(value, dict):
            filtered = filter_results(value)
            if len(results) > 0 and len(filtered) > 0:
                results[0][key] = filtered[0]
            if len(results) > 0 and len(filtered) > 1:
                # if the value has derived 'STANDALONE' instances, extend them
                results.extend(filtered[1:])
        elif isinstance(value, list):
            filtered_list = []
            standalone_list = []
            ignore_schema_count = 0
            for i in value:
                if is_kcl_schema(i):
                    filtered, standalone = handle_schema(i)
                    if filtered is None:
                        ignore_schema_count += 1
                        continue
                    if filtered:
                        if standalone:
                            standalone_list.extend(filtered)
                        else:
                            filtered_list.extend(filtered)
                elif isinstance(i, dict):
                    filtered = filter_results(i)
                    if filtered:
                        filtered_list.extend(filtered)
                elif i is None and kclvm.config.disable_none:
                    continue
                elif not isinstance(i, UndefinedType):
                    # Filter list elements
                    filtered = filter_results({LIST_DICT_TEMP_KEY: i})
                    if (
                        len(results) > 0
                        and len(filtered) > 0
                        and LIST_DICT_TEMP_KEY in filtered[0]
                    ):
                        filtered_list.append(filtered[0][LIST_DICT_TEMP_KEY])
                    if len(results) > 0 and len(filtered) > 1:
                        # if the value has derived 'STANDALONE' instances, extend them
                        results.extend(filtered[1:])
            schema_in_list_count = ignore_schema_count + len(standalone_list)
            if len(results) > 0 and 0 <= schema_in_list_count < len(value):
                results[0][key] = filtered_list
            if standalone_list:
                results.extend(standalone_list)
        else:
            results[0][key] = value
    return results


@dataclass
class ObjectPlanner:
    """Planner is used to modify VM exec result to YAML"""

    def __init__(
        self, *, sort_keys: bool = False, include_schema_type_path: bool = False
    ) -> None:
        self.sort_keys = sort_keys
        self.include_schema_type_path = include_schema_type_path

    def to_python(self, v):
        if isinstance(v, obj.KCLObject):
            if isinstance(v, obj.KCLLiteralObject):
                return v.value
            elif isinstance(v, obj.KCLSchemaObject):
                result = {_k: self.to_python(_v) for _k, _v in v.attrs.items()}
                if self.include_schema_type_path:
                    result["@type"] = v.full_type_str()
                return result
            elif isinstance(v, (obj.KCLListObject, obj.KCLTupleObject)):
                return [self.to_python(_v) for _v in v.value]
            elif isinstance(v, obj.KCLDictObject):
                return {_k: self.to_python(_v) for _k, _v in v.value.items()}
            elif isinstance(v, (obj.KCLUndefinedObject, obj.KCLFunctionObject)):
                return Undefined
            elif isinstance(v, obj.KCLNameConstantObject):
                return v.value
            else:
                return Undefined
        elif isinstance(v, (list, tuple, set)):
            return [self.to_python(_v) for _v in v]
        elif isinstance(v, dict):
            return {_k: self.to_python(_v) for _k, _v in v.items()}
        elif isinstance(v, (int, float, str, bool)) or v is None:
            return v
        elif v is Undefined or isinstance(v, UndefinedType):
            return v
        else:
            raise Exception("Invalid KCL Object")

    def plan(self, var_dict: Dict[str, obj.KCLObject]) -> Dict[str, any]:
        assert isinstance(var_dict, dict)
        data = {
            k: self.to_python(v)
            for k, v in var_dict.items()
            if v and v.type() in KCL_PLAN_TYPE
        }
        return data


@dataclass
class YAMLPlanner(ObjectPlanner):
    def __init__(
        self, *, sort_keys: bool = False, include_schema_type_path: bool = False
    ) -> None:
        super().__init__(
            sort_keys=sort_keys, include_schema_type_path=include_schema_type_path
        )

    def plan(self, var_dict: Dict[str, obj.KCLObject], to_py: bool = True) -> str:
        assert isinstance(var_dict, dict)
        plan_obj = super().plan(var_dict) if to_py else var_dict
        # Represent OrderedDict as dict.
        yaml = YAML()
        yaml.representer.add_representer(
            OrderedDict,
            lambda dumper, data: dumper.represent_mapping(
                "tag:yaml.org,2002:map", data
            ),
        )
        # Convert tuple to list.
        yaml.representer.add_representer(
            tuple,
            lambda dumper, data: dumper.represent_sequence(
                "tag:yaml.org,2002:seq", data
            ),
        )
        # Convert None to null
        yaml.representer.add_representer(
            type(None),
            lambda dumper, data: dumper.represent_scalar(
                u"tag:yaml.org,2002:null", u"null"
            ),
        )
        yaml.representer.add_representer(
            str,
            lambda dumper, data: dumper.represent_scalar(
                u"tag:yaml.org,2002:str", data, style="|"
            )
            if "\n" in data
            else dumper.represent_str(data),
        )
        results = filter_results(plan_obj)
        if self.sort_keys:
            results = [order_dict(r) for r in results]
        outputs = ""
        with io.StringIO() as buf:
            if results:
                for result in results[:-1]:
                    if result:
                        yaml.dump(result, buf)
                        buf.write("---\n")
                if results[-1]:
                    yaml.dump(results[-1], buf)
                outputs = buf.getvalue()
        return outputs


class JSONPlanner(ObjectPlanner):
    def __init__(
        self, *, sort_keys: bool = False, include_schema_type_path: bool = False
    ) -> None:
        super().__init__(
            sort_keys=sort_keys, include_schema_type_path=include_schema_type_path
        )

    def plan(
        self,
        var_dict: Dict[str, obj.KCLObject],
        *,
        only_first=False,
        to_py: bool = True
    ) -> str:
        assert isinstance(var_dict, dict)
        plan_obj = super().plan(var_dict) if to_py else var_dict
        results = filter_results(plan_obj)
        if self.sort_keys:
            results = [order_dict(r) for r in results]
        results = [r for r in results if r]
        return (
            json.dumps(
                results if not only_first else results[0],
                default=lambda o: o.__dict__,
                indent=4,
            )
            if results
            else ""
        )


def plan(
    val_map: Dict[str, obj.KCLObject],
    *,
    sort_keys: bool = False,
    include_schema_type_path: bool = False
) -> str:
    return YAMLPlanner(
        sort_keys=sort_keys, include_schema_type_path=include_schema_type_path
    ).plan(val_map)
