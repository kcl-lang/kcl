# Copyright 2021 The KCL Authors. All rights reserved.

from typing import Optional, Union, List, Dict, Any, Iterator, cast
from abc import abstractmethod
from copy import deepcopy
from dataclasses import dataclass, field
from enum import Enum, IntEnum

import kclvm.kcl.error as kcl_error
import kclvm.kcl.info as kcl_info
import kclvm.kcl.ast as ast

from kclvm.api.object.internal import Deprecated, Undefined, UndefinedType

# --------------------------------
# KCL Object definitions
# --------------------------------

AttrType = str


class KCLObjectType(Enum):
    LITERAL = "LITERAL"
    INTEGER = "INTEGER"
    FLOAT = "FLOAT"
    BOOLEAN = "BOOLEAN"
    NONE = "NONE"
    UNDEFINED = "UNDEFINED"
    RETURN_VALUE = "RETURN_VALUE"
    ERROR = "ERROR"
    FUNCTION = "FUNCTION"
    STRING = "STRING"
    BUILTIN = "BUILTIN"
    LIST = "LIST"
    DICT = "DICT"
    NUMBER_MULTIPLIER = "NUMBER_MULTIPLIER"
    TUPLE = "TUPLE"
    ITER = "ITER"
    COMPILED_FUNCTION = "COMPILED_FUNCTION"
    CLOSURE = "CLOSURE"
    DECORATOR = "DECORATOR"
    SLICE = "SLICE"
    TYPE = "TYPE"
    SCHEMA_TYPE = "SCHEMA_TYPE"
    SCHEMA = "SCHEMA"
    SCHEMA_INDEX_SIGNATURE = "SCHEMA_INDEX_SIGNATURE"
    NAME_CONSTANT = "NAME_CONSTANT"
    MODULE = "MODULE"
    PACKAGE = "PACKAGE"
    UNPACK = "UNPACK"
    SCHEMA_CONFIG = "SCHEMA_CONFIG"
    RUNTIME_CODE = "RUNTIME_CODE"


class KCLSchemaReverseFields:
    SETTINGS = "__settings__"
    NAME = "__schema_name__"
    TYPE = "__schema_type__"
    PKG_PATH = "__pkg_path__"


NUMBER_MULTIPLIER_TYPE_STR = "number_multiplier"
ANT_TYPE_STR = "any"


@dataclass
class KCLObject:
    @abstractmethod
    def type(self) -> KCLObjectType:
        """
        Get the object type
        """
        pass

    @abstractmethod
    def type_str(self) -> str:
        """
        Get the object type
        """
        pass

    def is_truthy(self) -> bool:
        tpe = self.type()
        if tpe in [KCLObjectType.BOOLEAN, KCLObjectType.NONE]:
            return self.value
        elif tpe == KCLObjectType.UNDEFINED:
            return False
        elif tpe in [
            KCLObjectType.LIST,
            KCLObjectType.DICT,
            KCLObjectType.SCHEMA_TYPE,
            KCLObjectType.STRING,
        ]:
            return bool(self.value)
        elif tpe == KCLObjectType.INTEGER:
            return self.value != 0
        elif tpe == KCLObjectType.FLOAT:
            return self.value != 0.0
        return True


@dataclass
class KCLLiteralObject(KCLObject):
    value: Any

    def type(self) -> KCLObjectType:
        """
        Get the object type
        """
        return KCLObjectType.LITERAL

    def type_str(self) -> str:
        """
        Get the object type
        """
        return "literal"


@dataclass
class KCLIntObject(KCLLiteralObject):
    def type(self) -> KCLObjectType:
        """
        Get the object type
        """
        return KCLObjectType.INTEGER

    def type_str(self) -> str:
        """
        Get the object type
        """
        return "int"


@dataclass
class KCLFloatObject(KCLLiteralObject):
    def type(self) -> KCLObjectType:
        """
        Get the object type
        """
        return KCLObjectType.FLOAT

    def type_str(self) -> str:
        """
        Get the object type
        """
        return "float"


@dataclass
class KCLNumberMultiplierObject(KCLFloatObject):
    raw_value: int
    binary_suffix: str

    def type(self) -> KCLObjectType:
        """
        Get the object type
        """
        return KCLObjectType.NUMBER_MULTIPLIER

    def type_str(self) -> str:
        """
        Get the object type
        """
        return (
            f"{NUMBER_MULTIPLIER_TYPE_STR}({self.raw_value}{self.binary_suffix})"
            if self.raw_value and self.binary_suffix
            else NUMBER_MULTIPLIER_TYPE_STR
        )

    def __str__(self) -> str:
        return f"{self.raw_value}{self.binary_suffix}"

    def __repr__(self) -> str:
        return self.__str__()

    def __int__(self) -> int:
        return int(self.value)

    def __float__(self) -> float:
        return float(self.value)

    def __bool__(self) -> bool:
        return bool(self.value)


@dataclass
class KCLStringObject(KCLLiteralObject):

    MEMBER_FUNCTIONS = [
        "capitalize",
        "count",
        "endswith",
        "find",
        "format",
        "index",
        "isalnum",
        "isalpha",
        "isdigit",
        "islower",
        "isspace",
        "istitle",
        "isupper",
        "join",
        "lower",
        "upper",
        "lstrip",
        "rstrip",
        "replace",
        "rfind",
        "rindex",
        "rsplit",
        "split",
        "splitlines",
        "startswith",
        "strip",
        "title",
    ]

    def type(self) -> KCLObjectType:
        """
        Get the object type
        """
        return KCLObjectType.STRING

    def type_str(self) -> str:
        """
        Get the object type
        """
        return "str"

    def check_attr(self, name: str):
        if not name:
            kcl_error.report_exception(
                err_type=kcl_error.ErrType.EvaluationError_TYPE,
                arg_msg="kcl string object member name can't be empty or None",
            )
        if not hasattr(self.value, name) and name not in self.MEMBER_FUNCTIONS:
            kcl_error.report_exception(
                err_type=kcl_error.ErrType.EvaluationError_TYPE,
                arg_msg=f"attribute {name} not found",
            )

    def get_member_method(self, name: str):
        from .function import KCLMemberFunctionObject

        self.check_attr(name)
        return KCLMemberFunctionObject(obj=self, name=name)

    def call_member_method(self, name: str, *args, **kwargs):

        return getattr(self.value, name).__call__(*args, **kwargs)


@dataclass
class KCLNameConstantObject(KCLObject):
    value: Any

    def type(self) -> KCLObjectType:
        """
        Get the object type
        """
        return KCLObjectType.NAME_CONSTANT

    def type_str(self) -> str:
        """
        Get the object type
        """
        return "name_constant"


@dataclass
class KCLTrueObject(KCLNameConstantObject):
    def type(self) -> KCLObjectType:
        """
        Get the object type
        """
        return KCLObjectType.BOOLEAN

    def type_str(self) -> str:
        """
        Get the object type
        """
        return "bool"

    @staticmethod
    def instance() -> KCLNameConstantObject:
        return TRUE_INSTANCE


@dataclass
class KCLFalseObject(KCLNameConstantObject):
    def type(self) -> KCLObjectType:
        """
        Get the object type
        """
        return KCLObjectType.BOOLEAN

    def type_str(self) -> str:
        """
        Get the object type
        """
        return "bool"

    @staticmethod
    def instance() -> KCLNameConstantObject:
        return FALSE_INSTANCE


@dataclass
class KCLNoneObject(KCLNameConstantObject):
    def type(self) -> KCLObjectType:
        """
        Get the object type
        """
        return KCLObjectType.NONE

    def type_str(self) -> str:
        """
        Get the object type
        """
        return "NoneType"

    @staticmethod
    def instance() -> "KCLNoneObject":
        return NONE_INSTANCE


@dataclass
class KCLUndefinedObject(KCLNameConstantObject):
    def type(self) -> KCLObjectType:
        """
        Get the object type
        """
        return KCLObjectType.UNDEFINED

    def type_str(self) -> str:
        """
        Get the object type
        """
        return Undefined.type_str()

    @staticmethod
    def instance() -> "KCLUndefinedObject":
        return UNDEFINED_INSTANCE


@dataclass
class KCLListObject(KCLObject):
    items: List[KCLObject] = field(default_factory=list)

    def type(self) -> KCLObjectType:
        """
        Get the object type
        """
        return KCLObjectType.LIST

    def type_str(self) -> str:
        """
        Get the object type
        """
        return "list"

    def append(self, item: KCLObject):
        self.items.append(item)

    def append_unpack(self, items: KCLObject):
        if not isinstance(
            items,
            (
                KCLListObject,
                KCLSchemaObject,
                KCLDictObject,
                KCLNoneObject,
                KCLUndefinedObject,
            ),
        ):
            kcl_error.report_exception(
                err_type=kcl_error.ErrType.EvaluationError_TYPE,
                arg_msg=f"'{items.type_str()}' object is not iterable",
            )
        if not isinstance(items, (KCLNoneObject, KCLUndefinedObject)):
            self.items = [*self.items, *items.value]

    @property
    def value(self):
        return self.items

    def remove_at(self, index: KCLIntObject):
        del self.items[to_python_obj(index)]

    def remove(self, val: KCLObject):
        self.items.remove(val)


@dataclass
class KCLDictObject(KCLObject):
    value: dict = field(default_factory=dict)

    def type(self) -> KCLObjectType:
        """
        Get the object type
        """
        return KCLObjectType.DICT

    def type_str(self) -> str:
        """
        Get the object type
        """
        return "dict"

    @property
    def config_keys(self) -> set:
        return set(self.value.keys())

    def update_key_value(self, k: AttrType, v: KCLObject):
        assert k is not None
        self.value[to_python_obj(k)] = to_kcl_obj(v)

    def has_key(self, k: AttrType):
        return to_python_obj(k) in self.value

    def __contains__(self, k: AttrType):
        return to_python_obj(k) in self.value

    def get(self, k: AttrType) -> KCLObject:
        if k is None or isinstance(k, UndefinedType):
            return KCLUndefinedObject.instance()
        if isinstance(k, (KCLNoneObject, KCLUndefinedObject)):
            return KCLUndefinedObject.instance()
        return self.value.get(to_python_obj(k), KCLUndefinedObject.instance())

    def get_keys(self) -> KCLListObject:
        return KCLListObject([to_kcl_obj(v) for v in self.value.keys()])

    def get_values(self) -> KCLListObject:
        return KCLListObject([to_kcl_obj(v) for v in self.value.values()])

    def append_unpack(self, items: KCLObject):
        if not isinstance(
            items,
            (
                KCLListObject,
                KCLSchemaObject,
                KCLDictObject,
                KCLNoneObject,
                KCLUndefinedObject,
            ),
        ):
            kcl_error.report_exception(
                err_type=kcl_error.ErrType.EvaluationError_TYPE,
                arg_msg=f"'{items.type_str()}' object is not iterable",
            )
        value = (
            KCLDictObject(value={})
            if isinstance(items, (KCLNoneObject, KCLUndefinedObject))
            else items
        )
        if isinstance(value, KCLSchemaObject):
            config = KCLSchemaConfigObject(value=value.value)
            config.update_attr_op_using_obj(value)
            self.union_with(deepcopy(config))
        else:
            self.union_with(deepcopy(value))

    def update(self, data: Union[dict, "KCLDictObject"]):
        if isinstance(data, KCLDictObject):
            for k, v in data.value.items():
                self.value[k] = to_kcl_obj(v)
        if isinstance(data, dict):
            for k, v in data.items():
                self.value[k] = to_kcl_obj(v)

    def union_with(self, obj: KCLObject, should_idempotent_check: bool = True):
        from kclvm.vm.runtime.evaluator import union

        union(self, to_kcl_obj(obj), should_idempotent_check=should_idempotent_check)
        if isinstance(self, KCLConfigObjectMixin):
            self.update_attr_op_using_obj(obj)

    def merge_with(self, obj: KCLObject):
        from kclvm.vm.runtime.evaluator import merge

        union_obj = cast(
            KCLDictObject,
            merge([self, obj]),
        )
        self.update(union_obj)

    def insert_with_key(
        self, attr: Union[str, KCLStringObject], obj: KCLObject, index=-1
    ):
        value = self.get(attr)
        if (
            value is None
            or value is Undefined
            or isinstance(value, UndefinedType)
            or isinstance(value, (KCLNoneObject, KCLUndefinedObject))
        ):
            value = KCLListObject()
            self.value[attr] = value
        if not isinstance(value, KCLListObject) or not isinstance(obj, KCLListObject):
            kcl_error.report_exception(
                err_type=kcl_error.ErrType.EvaluationError_TYPE,
                arg_msg="only list attribute can be inserted value",
            )
        if index is None or index == -1:
            value = self.value[attr].value + obj.value
            self.update({attr: value})
        elif index >= 0:
            value = (
                self.value[attr].value[:index]
                + obj.value
                + self.value[attr].value[index:]
            )
            self.update({attr: value})

    def insert_with(
        self, data: Union[dict, "KCLDictObject"], index: Optional[int] = None
    ):
        obj = data.value if isinstance(data, KCLDictObject) else data
        if not isinstance(obj, dict):
            return
        for k, v in obj.items():
            self.insert_with_key(k, v, index)

    def list_key_override(self, attr: str, v: KCLObject, index: int):
        value = self.get(attr)
        if not isinstance(value, KCLListObject):
            kcl_error.report_exception(
                err_type=kcl_error.ErrType.EvaluationError_TYPE,
                arg_msg="only list attribute can be inserted value",
            )
        if v is None or isinstance(v, (KCLNoneObject, KCLUndefinedObject)):
            self.value[attr].value.pop(index)
        else:
            self.value[attr].value[index] = v

    def unique_merge_with(self, obj: KCLObject):
        from kclvm.vm.runtime.evaluator import union

        union(self, to_kcl_obj(obj), should_idempotent_check=True)

    def delete(self, key: Union[str, KCLStringObject]):
        del self.value[to_python_obj(key)]


@dataclass
class KCLConfigObjectMixin:
    operation_map: Dict[str, int] = field(default_factory=dict)
    insert_index_map: Dict[str, Union[str, int]] = field(default_factory=dict)

    def add_operation(self, key: str, operation: int, insert_index=-1):
        if not self.operation_map:
            self.operation_map = {}
        if not self.insert_index_map:
            self.insert_index_map = {}
        if not key:
            return
        self.operation_map[key] = operation
        self.insert_index_map[key] = insert_index

    def get_operation(self, key: str) -> int:
        if not self.operation_map:
            self.operation_map = {}
        return self.operation_map.get(key, ast.ConfigEntryOperation.UNION)

    def get_insert_index(self, key: str) -> Optional[Union[str, int]]:
        if not self.insert_index_map:
            return None
        return self.insert_index_map.get(key)

    def update_attr_op_using_obj(self, obj: KCLObject):
        if isinstance(obj, (KCLSchemaConfigObject, KCLSchemaObject)):
            self.operation_map = {**self.operation_map, **obj.operation_map}
            self.insert_index_map = {**self.insert_index_map, **obj.insert_index_map}


@dataclass
class KCLSchemaConfigObject(KCLDictObject, KCLConfigObjectMixin):
    def type(self) -> KCLObjectType:
        """
        Get the object type
        """
        return KCLObjectType.DICT

    def type_str(self) -> str:
        return "dict"  # Please note it is actually a dict


@dataclass
class KCLSchemaObject(KCLObject, KCLConfigObjectMixin):
    name: str = None
    pkgpath: str = None
    instance_pkgpath: str = None
    attrs: dict = None
    runtime_type: str = None
    is_relaxed: bool = False
    config_keys: set = field(default_factory=set)
    __tags__: dict = field(default_factory=dict)
    __decorators__: dict = field(default_factory=dict)
    __stmt_buffer__: list = field(default_factory=dict)

    def type(self) -> KCLObjectType:
        """
        Get the object type
        """
        return KCLObjectType.SCHEMA

    def type_str(self) -> str:
        """
        Get the object type
        """
        return self.name

    def full_type_str(self) -> str:
        """
        Get the object type
        """
        return (
            f"{self.pkgpath}.{self.name}"
            if self.pkgpath and self.pkgpath != ast.Program.MAIN_PKGPATH
            else self.name
        )

    @property
    def value(self):
        return self.attrs

    def update_info(self, name: str, runtime_type: str, is_relaxed: bool):
        self.name = name
        self.runtime_type = runtime_type
        self.is_relaxed = is_relaxed

    def construct(
        self, config: Optional[KCLDictObject] = None, _args=None, _builder=None
    ):
        if config and isinstance(config, KCLDictObject):
            self.attrs = config.value
        return self

    def update(self, data: Union[dict, KCLObject]):
        if isinstance(data, KCLDictObject):
            for k, v in data.value.items():
                self.attrs[to_python_obj(k)] = to_kcl_obj(v)
        if isinstance(data, dict):
            for k, v in data.items():
                self.attrs[to_python_obj(k)] = to_kcl_obj(v)

    def get(self, k: Union[str, KCLStringObject], do_check: bool = True):
        if not self.attrs or k not in self.attrs and do_check:
            kcl_error.report_exception(
                err_type=kcl_error.ErrType.AttributeError_TYPE,
                arg_msg=f"schema '{self.full_type_str()}' attribute '{k}' not found",
            )
        return self.attrs.get(to_python_obj(k))

    def update_key_value(self, k: str, v: KCLObject):
        assert k is not None
        self.attrs[to_python_obj(k)] = to_kcl_obj(v)

    def append_unpack(self, items: KCLObject):
        if not isinstance(
            items,
            (
                KCLListObject,
                KCLSchemaObject,
                KCLDictObject,
                KCLNoneObject,
                KCLUndefinedObject,
            ),
        ):
            kcl_error.report_exception(
                err_type=kcl_error.ErrType.EvaluationError_TYPE,
                arg_msg=f"'{items.type_str()}' object is not iterable",
            )
        value = (
            KCLDictObject(value={})
            if isinstance(items, (KCLNoneObject, KCLUndefinedObject))
            else items
        )
        if isinstance(value, KCLSchemaObject):
            config = KCLSchemaConfigObject(value=value.value)
            config.update_attr_op_using_obj(value)
            self.union_with(config)
        else:
            self.union_with(value)

    def insert_with_key(
        self, attr: Union[str, KCLStringObject], obj: KCLObject, index=-1
    ):
        self.insert_with(attr, obj, index)

    def insert_with(self, attr: Union[str, KCLStringObject], obj: KCLObject, index=-1):
        value = self.get(attr, do_check=False)
        if (
            value is None
            or value is Undefined
            or isinstance(value, UndefinedType)
            or isinstance(value, (KCLNoneObject, KCLUndefinedObject))
        ):
            value = KCLListObject()
            self.attrs[attr] = value
        if not isinstance(value, KCLListObject) or not isinstance(obj, KCLListObject):
            kcl_error.report_exception(
                err_type=kcl_error.ErrType.EvaluationError_TYPE,
                arg_msg="only list attribute can be inserted value",
            )
        if index is None or index == -1:
            value = self.attrs[attr].value + obj.value
            self.update({attr: value})
        elif index >= 0:
            value = (
                self.attrs[attr].value[:index]
                + obj.value
                + self.attrs[attr].value[index:]
            )
            self.update({attr: value})

    def list_key_override(self, attr: str, v: KCLObject, index: int):
        value = self.get(attr)
        if not isinstance(value, KCLListObject):
            kcl_error.report_exception(
                err_type=kcl_error.ErrType.EvaluationError_TYPE,
                arg_msg="only list attribute can be inserted value",
            )
        if v is None or isinstance(v, (KCLNoneObject, KCLUndefinedObject)):
            self.attrs[attr].value.pop(index)
        else:
            self.attrs[attr].value[index] = v

    def union_with(
        self, obj: Union[KCLObject, dict], should_idempotent_check: bool = True
    ):
        from kclvm.vm.runtime.evaluator import union

        union(self, to_kcl_obj(obj), should_idempotent_check=should_idempotent_check)
        if isinstance(self, KCLConfigObjectMixin):
            self.update_attr_op_using_obj(obj)

    def delete(self, key: Union[str, KCLStringObject]):
        del self.value[to_python_obj(key)]

    def has_key(self, attr: Union[str, KCLStringObject]):
        return to_python_obj(attr) in self.attrs

    def __contains__(self, attr: Union[str, KCLStringObject]):
        return to_python_obj(attr) in self.attrs

    def should_add_attr(self, name: str) -> bool:
        """Determine whether an attribute can be added to schema attributes,
        such as non-exported variables that start with `_` or relaxed attributes.

        Three situations that should be added:
        1. The attribute was originally an attribute of this schema
        2. Variables starting with an underscore `_`
        3. The schema is relaxed
        """
        return (
            name in self
            and self.get_attr_type(name)
            or (isinstance(name, str) and name.startswith("_"))
            or self.is_relaxed
        )

    # Attribute type

    def set_attr_type(self, attr: str, types: List[str]):
        if not attr:
            return
        tagged = kcl_info.tagging("attr_type", attr)
        if not self.__tags__:
            self.__tags__ = {}
        self.__tags__[tagged] = types

    def get_attr_type(self, attr: str) -> List[str]:

        if not attr or not self.__tags__:
            return []
        tagged = kcl_info.tagging("attr_type", attr)
        if tagged in self.__tags__:
            return self.__tags__[tagged]
        else:
            return []

    def set_attr_runtime_type(self, attr: str, types: List[str]):
        if not attr:
            return
        tagged = kcl_info.tagging("runtime_attr_type", attr)
        if not self.__tags__:
            self.__tags__ = {}
        if tagged in self.__tags__ and self.__tags__[tagged] != types:
            kcl_error.report_exception(
                err_type=kcl_error.ErrType.TypeError_Runtime_TYPE,
                arg_msg=f"can't change schema field type of '{attr}'",
            )
        self.__tags__[tagged] = types

    # Optional

    def set_attr_optional(self, attr: str, is_optional: bool):
        if not attr:
            return
        tagged = kcl_info.tagging("is_optional", attr)
        if not self.__tags__:
            self.__tags__ = {}
        if tagged in self.__tags__:
            if self.__tags__[tagged] is False and is_optional is True:
                kcl_error.report_exception(
                    err_type=kcl_error.ErrType.EvaluationError_TYPE,
                    arg_msg=f"can't change the required schema attribute of '{attr}' to optional",
                )
        self.__tags__[tagged] = is_optional

    def get_attr_optional(self, attr: str) -> bool:
        if not attr:
            return False
        tagged = kcl_info.tagging("is_optional", attr)
        if not self.__tags__:
            self.__tags__ = {}
        return self.__tags__.get(tagged, False)

    def check_optional_attrs(self):
        """Check all schema attributes are optional.
        If the schema attribute is not optional and its value is None, an error is reported
        """
        if not self.__tags__:
            return
        for k, v in self.attrs.items():
            # Note k is a string, v is a KCLObject
            is_optional = self.get_attr_optional(str(k))
            # Relaxed schema attribute has no types and do not check the None value
            types = self.get_attr_type(k)
            # Whether v is not a optional attribute
            if (
                types
                and not is_optional
                and (v is None or isinstance(v, (KCLNoneObject, KCLUndefinedObject)))
            ):
                kcl_error.report_exception(
                    err_type=kcl_error.ErrType.EvaluationError_TYPE,
                    arg_msg=f"attribute '{k}' of {self.name} is required and can't be None or Undefined",
                )

    # Attribute mutable

    def set_immutable_flag(self, attr: str, is_final: bool):
        if not attr:
            return
        tagged = kcl_info.tagging("immutable", attr)
        if not self.__tags__:
            self.__tags__ = {}
        self.__tags__[tagged] = is_final

    def get_immutable_flag(self, attr: str) -> bool:
        if not attr:
            return False
        tagged = kcl_info.tagging("immutable", attr)
        if not self.__tags__:
            self.__tags__ = {}
        return self.__tags__.get(tagged, False)

    # Decorators

    def add_decorator(self, field: str, decorator) -> None:
        """
        Add a decorator to the schema

        Parameters
        ----------
        - field: The schema attribute name or schema name
        - decorator: A decorator class

        Return
        ------
        None
        """
        if not field:
            return
        if not self.__decorators__:
            self.__decorators__ = {}
        tagged = kcl_info.tagging("decorator", field)
        if tagged not in self.__decorators__:
            self.__decorators__[tagged] = []
        self.__decorators__[tagged].append(decorator)

    def run_all_decorators(self) -> None:
        """
        Run schema all decorators and
        parameters of per decorator is its key and value.
        """
        if not self.__decorators__:
            return
        for k in self.__decorators__:
            for decorator in self.__decorators__[k]:
                name_member = kcl_info.detagging("decorator", k)
                value = decorator.call(
                    None,
                    None,
                    key=name_member,
                    value=self.attrs.get(name_member),
                )
                # If a schema attribute is deprecated, RESET it to be None
                if (
                    not value
                    or isinstance(value, (KCLNoneObject, KCLUndefinedObject))
                    and decorator.name == Deprecated.NAME
                ):
                    self.attrs[name_member] = None

    # Statement buffer

    def stmt_buffer_enqueue(self, content):
        if not self.__stmt_buffer__:
            self.__stmt_buffer__ = []
        if content and hasattr(self, "__stmt_buffer__"):
            self.__stmt_buffer__.append(content)

    def stmt_buffer(self):
        if hasattr(self, "__stmt_buffer__"):
            return self.__stmt_buffer__
        return None


@dataclass
class KCLUnpackObject(KCLObject):
    value: Union[
        KCLListObject, KCLDictObject, KCLSchemaObject, KCLNoneObject, KCLUndefinedObject
    ] = None
    is_double_star: bool = False

    def type(self) -> KCLObjectType:
        """
        Get the object type
        """
        return KCLObjectType.UNPACK

    def type_str(self) -> str:
        """
        Get the object type
        """
        return "unpack"

    def unpack(self):
        if not isinstance(
            self.value,
            (
                KCLListObject,
                KCLDictObject,
                KCLSchemaObject,
                KCLNoneObject,
                KCLUndefinedObject,
            ),
        ):
            kcl_error.report_exception(
                err_type=kcl_error.ErrType.EvaluationError_TYPE,
                arg_msg=f"only list, dict and schema object can be used with unpack operators * and **, got {self.value}",
            )
        return self.value


@dataclass
class KCLSliceObject(KCLObject):
    start: KCLObject = None
    stop: KCLObject = None
    step: KCLObject = None

    def type(self) -> KCLObjectType:
        """
        Get the object type
        """
        return KCLObjectType.SLICE

    def type_str(self) -> str:
        """
        Get the object type
        """
        return "slice"

    @property
    def value(self):
        return slice(self.start.value, self.stop.value, self.step.value)


@dataclass
class KCLModuleObject(KCLObject):
    name: str
    asname: str = None
    value: Dict[str, KCLObject] = None

    def type(self) -> KCLObjectType:
        """
        Get the object type
        """
        return KCLObjectType.MODULE

    def type_str(self) -> str:
        """
        Get the object type
        """
        return "module"

    def get(self, name: str):
        name = to_python_obj(name)
        if self.value and name and name in self.value:
            return self.value[name]
        kcl_error.report_exception(
            err_type=kcl_error.ErrType.AttributeError_Runtime_TYPE,
            arg_msg=f"module '{self.name}' has no attribute '{name}'",
        )


@dataclass
class KCLIterObject(KCLObject):
    iter: Iterator

    def type(self) -> KCLObjectType:
        """
        Get the object type
        """
        return KCLObjectType.ITER

    def type_str(self) -> str:
        """
        Get the object type
        """
        return "iter"

    def next(self) -> KCLObject:
        next_obj = next(self.iter)
        return to_kcl_obj(next_obj if isinstance(next_obj, tuple) else (next_obj,))

    @staticmethod
    def build_iter(obj: KCLObject, iter_variable_count: int = 1):
        if obj.type() not in [
            KCLObjectType.DICT,
            KCLObjectType.LIST,
            KCLObjectType.STRING,
            KCLObjectType.SCHEMA,
        ]:
            kcl_error.report_exception(
                err_type=kcl_error.ErrType.EvaluationError_TYPE,
                arg_msg=f"{obj.type_str()} object is not iterable",
            )
        assert 0 < iter_variable_count <= 2
        if iter_variable_count == 1:
            return KCLIterObject(iter=iter(obj.value))
        if obj.type() in [KCLObjectType.LIST, KCLObjectType.STRING]:
            return KCLIterObject(iter=iter(enumerate(obj.value)))
        if obj.type() in [KCLObjectType.DICT, KCLObjectType.SCHEMA]:
            return KCLIterObject(iter=iter(obj.value.items()))
        raise Exception(f"invalid iter object type {type(obj)}")

    @property
    def value(self):
        return self.iter


@dataclass
class KCLTupleObject(KCLObject):
    value: List[KCLObject]

    def type(self) -> KCLObjectType:
        """
        Get the object type
        """
        return KCLObjectType.TUPLE

    def type_str(self) -> str:
        """
        Get the object type
        """
        return "tuple"


@dataclass
class KCLErrorObject(KCLObject, Exception):
    file: Optional[str] = None
    lineno: Optional[int] = None
    colno: Optional[int] = None
    msg: str = None

    def type(self) -> KCLObjectType:
        """
        Get the object type
        """
        return KCLObjectType.ERROR

    def type_str(self) -> str:
        """
        Get the object type
        """
        return "error"

    @property
    def value(self):
        return self.msg


# ----------------------
# KCL type objects
# ----------------------


class KCLTypeKind(IntEnum):
    NoneKind = 0
    AnyKind = 1
    UnionKind = 2
    BoolKind = 3
    BoolLitKind = 4
    IntKind = 5
    IntLitKind = 6
    FloatKind = 7
    FloatLitKind = 8
    StrKind = 9
    StrLitKind = 10
    ListKind = 11
    DictKind = 12
    SchemaKind = 13
    SchemaDefKind = 14
    NumberMultiplierKind = 15
    FuncKind = 16
    VoidKind = 17
    ModuleKind = 18
    NamedKind = 19


class TypeAliasMixin:
    # Mark the type is a value or a type alias
    is_type_alias: bool = False


@dataclass
class KCLBaseTypeObject(KCLObject, TypeAliasMixin):
    def type(self) -> KCLObjectType:
        """
        Get the object type
        """
        return KCLObjectType.TYPE

    def type_str(self) -> str:
        """
        Get the object type
        """
        return "base_type"

    def type_kind(self) -> int:
        return -1


@dataclass
class KCLAnyTypeObject(KCLBaseTypeObject):
    value: str = None  # Can only be any

    def type_str(self) -> str:
        """
        Get the object type
        """
        return "any"

    def type_kind(self) -> int:
        return KCLTypeKind.AnyKind


@dataclass
class KCLBuiltinTypeObject(KCLBaseTypeObject):
    def type_str(self) -> str:
        """
        Get the object type
        """
        return "builtin"


@dataclass
class KCLIntTypeObject(KCLBuiltinTypeObject):
    def type_str(self):
        return "int"

    def type_kind(self) -> int:
        return KCLTypeKind.IntKind


@dataclass
class KCLFloatTypeObject(KCLBuiltinTypeObject):
    def type_str(self):
        return "float"

    def type_kind(self) -> int:
        return KCLTypeKind.FloatKind


@dataclass
class KCLStringTypeObject(KCLBuiltinTypeObject):
    def type_str(self):
        return "str"

    def type_kind(self) -> int:
        return KCLTypeKind.StrKind


@dataclass
class KCLBoolTypeObject(KCLBuiltinTypeObject):
    def type_str(self) -> str:
        """
        Get the object type
        """
        return "bool"

    def type_kind(self) -> int:
        return KCLTypeKind.BoolKind


@dataclass
class KCLNameConstantTypeObject(KCLBaseTypeObject):
    value: Optional[bool] = None

    def type_str(self) -> str:
        """
        Get the object type
        """
        return "NoneType" if self.value is None else "bool"


@dataclass
class KCLStringLitTypeObject(KCLBaseTypeObject):
    value: str = None

    def type_str(self) -> str:
        """
        Get the object type
        """
        return f"str({self.value})"

    def type_kind(self) -> int:
        return KCLTypeKind.StrLitKind


@dataclass
class KCLNumberLitTypeObject(KCLBaseTypeObject):
    value: Union[int, float] = None

    def type_str(self) -> str:
        """
        Get the object type
        """
        return (
            f"int({self.value})"
            if isinstance(self.value, int)
            else f"float({self.value})"
        )

    def type_kind(self) -> int:
        return (
            KCLTypeKind.IntLitKind
            if self.is_int_lit_type()
            else KCLTypeKind.FloatLitKind
        )

    def is_int_lit_type(self) -> bool:
        return isinstance(self.value, int)

    def is_float_lit_type(self) -> bool:
        return isinstance(self.value, float)


@dataclass
class KCLIntLitTypeObject(KCLNumberLitTypeObject):
    def type_kind(self) -> int:
        return KCLTypeKind.IntLitKind


@dataclass
class KCLFloatLitTypeObject(KCLNumberLitTypeObject):
    def type_kind(self) -> int:
        return KCLTypeKind.FloatLitKind


@dataclass
class KCLBoolLitTypeObject(KCLBaseTypeObject):
    value: bool = None

    def type_str(self) -> str:
        return f"bool({self.value})"

    def type_kind(self) -> int:
        return KCLTypeKind.BoolLitKind


@dataclass
class KCLListTypeObject(KCLBaseTypeObject):
    item_type: KCLBaseTypeObject = None

    def type_str(self) -> str:
        """
        Get the object type
        """
        return "[{}]".format(self.item_type.type_str())

    def type_kind(self) -> int:
        return KCLTypeKind.ListKind


@dataclass
class KCLDictTypeObject(KCLBaseTypeObject):
    key_type: KCLBaseTypeObject = None
    value_type: KCLBaseTypeObject = None

    def type_str(self) -> str:
        """
        Get the object type
        """
        return "{{{}:{}}}".format(self.key_type.type_str(), self.value_type.type_str())

    def type_kind(self) -> int:
        return KCLTypeKind.DictKind


@dataclass
class KCLUnionTypeObject(KCLBaseTypeObject):
    types: List[KCLBaseTypeObject] = None

    def type_str(self) -> str:
        """
        Get the object type
        """
        return "{}".format("|".join([t.type_str() for t in self.types]))

    def type_kind(self) -> int:
        return KCLTypeKind.UnionKind


@dataclass
class KCLModuleTypeObject(KCLBaseTypeObject):
    pkgpath: str = ""
    imported_filenames: List[str] = field(default_factory=list)
    is_user_module: bool = False
    is_system_module: bool = False
    is_plugin_module: bool = False

    def type_str(self) -> str:
        return "module"

    def type_kind(self) -> int:
        return KCLTypeKind.ModuleKind


@dataclass
class KCLNumberMultiplierTypeObject(KCLBaseTypeObject):
    value: int = None
    raw_value: int = None
    binary_suffix: str = None

    def type_str(self) -> str:
        return (
            f"{NUMBER_MULTIPLIER_TYPE_STR}({self.raw_value}{self.binary_suffix})"
            if self.raw_value and self.binary_suffix
            else NUMBER_MULTIPLIER_TYPE_STR
        )

    def type_kind(self) -> int:
        return KCLTypeKind.NumberMultiplierKind

    def is_literal(self) -> bool:
        return bool(self.binary_suffix)


@dataclass
class KCLNamedTypeObject(KCLBaseTypeObject):
    name: str

    def type_str(self) -> str:
        return "named"

    def type_kind(self) -> int:
        return KCLTypeKind.NamedKind


@dataclass
class KCLNoneTypeObject(KCLNameConstantTypeObject):
    def type_str(self) -> str:
        """
        Get the object type
        """
        return "NoneType"

    def type_kind(self) -> int:
        return KCLTypeKind.NoneKind


@dataclass
class KCLVoidTypeObject(KCLBaseTypeObject):
    def type_str(self) -> str:
        """
        Get the object type
        """
        return "void"

    def type_kind(self) -> int:
        return KCLTypeKind.VoidKind


# --------------------
# KCL Object instances
# --------------------


TRUE_INSTANCE = KCLTrueObject(value=True)
FALSE_INSTANCE = KCLFalseObject(value=False)
NONE_INSTANCE = KCLNoneObject(value=None)
UNDEFINED_INSTANCE = KCLUndefinedObject(value=Undefined.value)


def to_python_obj(v: Union[KCLObject, int, float, str, bool, list, dict]) -> Any:
    if isinstance(v, KCLObject):
        if isinstance(v, KCLUndefinedObject):
            return Undefined
        elif isinstance(v, KCLNumberMultiplierObject):
            return v
        elif isinstance(v, (KCLLiteralObject, KCLNameConstantObject, KCLSliceObject)):
            return v.value
        elif isinstance(v, (KCLDictObject, KCLSchemaObject)):
            return {_k: to_python_obj(_v) for _k, _v in v.value.items()}
        elif isinstance(v, (KCLListObject, KCLTupleObject)):
            return [to_python_obj(_v) for _v in v.value]
    elif v is None or v is Undefined or isinstance(v, UndefinedType):
        return v
    elif isinstance(v, list):
        return [to_python_obj(_v) for _v in v]
    elif isinstance(v, dict):
        return {_k: to_python_obj(_v) for _k, _v in v.items()}
    elif isinstance(v, (int, float, str, bool, dict, list)):
        return v
    else:
        raise Exception(f"invalid KCL object type {type(v)} to native object")


def to_kcl_obj(
    value: Union[KCLObject, int, float, str, bool, list, dict, tuple]
) -> KCLObject:
    if isinstance(value, KCLObject):
        return value
    if value is None:
        return KCLNoneObject.instance()
    if value is Undefined or isinstance(value, UndefinedType):
        return KCLUndefinedObject.instance()
    if isinstance(value, bool):
        return KCLTrueObject.instance() if value else KCLFalseObject.instance()
    elif isinstance(value, int):
        return KCLIntObject(value=value)
    elif isinstance(value, float):
        return KCLFloatObject(value=value)
    elif isinstance(value, str):
        return KCLStringObject(value=value)
    elif isinstance(value, tuple):
        return KCLTupleObject(value=[to_kcl_obj(v) for v in value])
    elif isinstance(value, (list, set)):
        return KCLListObject([to_kcl_obj(v) for v in value])
    elif isinstance(value, dict):
        if KCLSchemaReverseFields.SETTINGS in value and isinstance(
            value[KCLSchemaReverseFields.SETTINGS], (dict, KCLDictObject)
        ):
            return KCLSchemaObject(
                attrs={k: to_kcl_obj(v) for k, v in value.items()},
                name=value[KCLSchemaReverseFields.SETTINGS].get(
                    KCLSchemaReverseFields.NAME
                ),
                runtime_type=value[KCLSchemaReverseFields.SETTINGS].get(
                    KCLSchemaReverseFields.TYPE
                ),
                pkgpath=value[KCLSchemaReverseFields.SETTINGS].get(
                    KCLSchemaReverseFields.PKG_PATH
                ),
            )
        return KCLDictObject({k: to_kcl_obj(v) for k, v in value.items()})
    else:
        raise Exception(f"invalid native object type {type(value)} to KCL object")
