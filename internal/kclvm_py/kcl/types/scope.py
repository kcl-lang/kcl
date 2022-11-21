# Copyright 2020 The KCL Authors. All rights reserved.

from typing import List, Dict, Optional, cast
from dataclasses import dataclass, field

import kclvm.kcl.ast as ast
import kclvm.api.object as objpkg
import kclvm.api.object.internal as internal
import kclvm.compiler.extension.builtin.builtin as builtin
import kclvm.compiler.extension.plugin as plugin

from .type import (
    Type,
    ANY_TYPE,
    STR_TYPE,
    BOOL_TYPE,
    INT_TYPE,
    FLOAT_TYPE,
    LIST_ANY_TYPE,
    LIST_STR_TYPE,
    DICT_ANY_ANY_TYPE,
    ITERABLE_TYPE_STR,
    ITERABLE_TYPE,
    NUMBER_TYPE_STR,
    NUMBER_TYPE,
)


@dataclass
class ScopeObject:
    """The object stored in the scope

    Parameters
    ----------
    name: str
        The scope object name.
    node: ast.AST
        The scope object AST node reference.
    type: Type
        The type of the scope object.
    pos: ast.Position
        The scope object start position.
    end: ast.Position
        The scope object end position.
    """

    name: str
    node: Optional[ast.AST]
    type: Type
    pos: ast.Position = None
    end: ast.Position = None

    def check_pos_valid(self):
        return (
            self.node
            and self.node.pos
            and self.node.pos.is_valid()
            and self.node.end_pos
            and self.node.end_pos.is_valid()
        )


@dataclass
class Scope:
    """A Scope maintains a set of objects and links to its containing
    (parent) and contained (children) scopes. Objects may be inserted
    and looked up by name. The zero value for Scope is a ready-to-use
    empty scope.

    Parameters
    ----------
    parent: Scope
        The parent scope.
    node: ast.AST
        The scope AST node reference.
    children: Type:
        The child scope list.
    elems: Dict[str, ScopeObject]
        The scope object mapping with its name.
    pos: ast.Position
        The scope start position.
    end: ast.Position
        The scope end position.
    """

    parent: "Scope" = None
    node: ast.AST = None
    children: List["Scope"] = field(default_factory=list)
    elems: Dict[str, ScopeObject] = field(default_factory=dict)
    pos: ast.Position = None
    end: ast.Position = None

    def contains_pos(self, pos: ast.Position) -> bool:
        """
        check if current scope contains a position
        :param pos: the given position
        :return: if current scope contains the given position
        """
        if isinstance(self.node, ast.SchemaStmt):
            for item in [
                *(self.node.body if self.node.body else []),
                *(self.node.checks if self.node.checks else []),
                self.node.index_signature,
            ]:
                if item and item.contains_pos(pos):
                    return True
            return False
        elif isinstance(self.node, ast.RuleStmt):
            for item in self.node.checks or []:
                if item and item.contains_pos(pos):
                    return True
            return False
        return self.pos and self.pos.less_equal(pos) and pos.less_equal(self.end)

    def inner_most(self, pos: ast.Position) -> Optional["Scope"]:
        # self is BUILTIN_SCOPE
        if self.parent is None:
            for child in self.children or []:
                if child.contains_pos(pos):
                    return child.inner_most(pos)
            return None
        # self is not BUILTIN_SCOPE
        if self.contains_pos(pos):
            for child in self.children or []:
                if child.contains_pos(pos):
                    return child.inner_most(pos)
            return self
        return None

    def get_enclosing_scope(self) -> Optional["Scope"]:
        return self.parent

    def get_parent_schema_scope(
        self, program_scope: "ProgramScope"
    ) -> Optional["Scope"]:
        if (
            self.node
            and isinstance(self.node, ast.SchemaStmt)
            and self.node.parent_name
        ):
            schema = self.parent.elems[self.node.name]
            if not isinstance(schema.type, objpkg.KCLSchemaDefTypeObject):
                return None
            schema_type_def_obj = cast(objpkg.KCLSchemaDefTypeObject, schema.type)
            if (
                not schema_type_def_obj.schema_type
                or not schema_type_def_obj.schema_type.base
            ):
                return None
            base_type_obj = schema_type_def_obj.schema_type.base
            # the schema and its base schema are in the same scope
            if schema_type_def_obj.schema_type.pkgpath == base_type_obj.pkgpath:
                return self.parent.search_child_scope_by_name(base_type_obj.name)
            # the schema and its base schema are in the different scopes
            base_pkg_scope = program_scope.scope_map.get(base_type_obj.pkgpath)
            if not base_pkg_scope:
                return None
            return base_pkg_scope.search_child_scope_by_name(base_type_obj.name)
        return None

    def search_child_scope_by_name(self, name: str) -> Optional["Scope"]:
        assert name
        if name not in self.elems:
            return None
        elem: ScopeObject = self.elems[name]
        for child in self.children:
            if child.node == elem.node:
                return child
        return None


@dataclass
class PackageScope(Scope):
    file_begin_position_map: Dict[str, ast.Position] = field(default_factory=dict)
    file_end_position_map: Dict[str, ast.Position] = field(default_factory=dict)

    def contains_pos(self, pos: ast.Position) -> bool:
        """
        check if current package scope contains a position
        :param pos: the given position
        :return: if current package scope contains the given position
        """
        assert pos.filename
        file_begin_pos = self.file_begin_position_map.get(pos.filename)
        file_end_pos = self.file_end_position_map.get(pos.filename)
        return (
            file_begin_pos
            and file_end_pos
            and pos
            and file_begin_pos.less_equal(pos)
            and pos.less_equal(file_end_pos)
        )


# Decorator scope including decorator function type such as `deprecated`.
DECORATOR_SCOPE: Scope = Scope(
    elems={
        internal.Deprecated.NAME: ScopeObject(
            name=internal.Deprecated.NAME,
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name=internal.Deprecated.NAME,
                params=[
                    objpkg.Parameter(
                        name="version",
                        type_annotation="str",
                        type=STR_TYPE,
                    ),
                    objpkg.Parameter(
                        name="reason",
                        type_annotation="str",
                        type=STR_TYPE,
                    ),
                    objpkg.Parameter(
                        name="strict",
                        type_annotation="bool",
                        type=BOOL_TYPE,
                    ),
                ],
                self_type=ANY_TYPE,
                return_type=ANY_TYPE,
                doc=internal.Deprecated.__doc__,
            ),
        ),
        internal.Info.NAME: ScopeObject(
            name=internal.Info.NAME,
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name=internal.Info.NAME,
                params=[],
                self_type=ANY_TYPE,
                return_type=ANY_TYPE,
                doc=internal.Info.__doc__,
            ),
        ),
    }
)


# Builtin-function types table
BUILTIN_SCOPE: Scope = Scope(
    elems={
        # option(key: str, *, type: str = "", required: bool = False, default: any = None, help: str = "", file: str = "", line: int = 0) -> any
        "option": ScopeObject(
            name="option",
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="option",
                params=[
                    objpkg.Parameter(
                        name="key",
                        value=None,
                        type_annotation="str",
                        type=STR_TYPE,
                    ),
                    objpkg.Parameter(
                        name="type",
                        value=objpkg.KCLStringObject(""),
                        type_annotation="str",
                        type=STR_TYPE,
                    ),
                    objpkg.Parameter(
                        name="required",
                        value=objpkg.KCLFalseObject.instance(),
                        type_annotation="bool",
                        type=BOOL_TYPE,
                    ),
                    objpkg.Parameter(
                        name="default",
                        value=objpkg.KCLNoneObject.instance(),
                        type_annotation="any",
                        type=ANY_TYPE,
                    ),
                    objpkg.Parameter(
                        name="help",
                        value=objpkg.KCLStringObject(""),
                        type_annotation="str",
                        type=STR_TYPE,
                    ),
                    objpkg.Parameter(
                        name="help",
                        value=objpkg.KCLStringObject(""),
                        type_annotation="str",
                        type=STR_TYPE,
                    ),
                    objpkg.Parameter(
                        name="file",
                        value=objpkg.KCLStringObject(""),
                        type_annotation="str",
                        type=STR_TYPE,
                    ),
                    objpkg.Parameter(
                        name="line",
                        value=objpkg.KCLIntObject(0),
                        type_annotation="int",
                        type=INT_TYPE,
                    ),
                ],
                self_type=ANY_TYPE,
                return_type=ANY_TYPE,
                doc=builtin.KMANGLED_option.__doc__,
            ),
        ),
        # print(value, ..., sep=' ', end='\n') -> NONE_TYPE
        "print": ScopeObject(
            name="print",
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="print",
                params=[],
                self_type=ANY_TYPE,
                return_type=ANY_TYPE,
                doc=builtin.KMANGLED_print.__doc__,
                is_variadic=True,
            ),
        ),
        # multiplyof(a: int, b: int) -> bool
        "multiplyof": ScopeObject(
            name="multiplyof",
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="multiplyof",
                params=[
                    objpkg.Parameter(
                        name="a",
                        value=None,
                        type_annotation="int",
                        type=INT_TYPE,
                    ),
                    objpkg.Parameter(
                        name="b",
                        value=None,
                        type_annotation="int",
                        type=INT_TYPE,
                    ),
                ],
                self_type=ANY_TYPE,
                return_type=INT_TYPE,
                doc=builtin.KMANGLED_multiplyof.__doc__,
            ),
        ),
        # isunique(inval: List[Any]) -> bool
        "isunique": ScopeObject(
            name="isunique",
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="isunique",
                params=[
                    objpkg.Parameter(
                        name="inval",
                        value=None,
                        type_annotation="[]",
                        type=LIST_ANY_TYPE,
                    ),
                ],
                self_type=ANY_TYPE,
                return_type=BOOL_TYPE,
                doc=builtin.KMANGLED_isunique.__doc__,
            ),
        ),
        # len(inval: Union[dict, list, schema, str]) -> int
        "len": ScopeObject(
            name="len",
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="len",
                params=[
                    objpkg.Parameter(
                        name="inval",
                        value=None,
                        type_annotation=ITERABLE_TYPE_STR,
                        type=ITERABLE_TYPE,
                    ),
                ],
                self_type=ANY_TYPE,
                return_type=INT_TYPE,
                doc=builtin.KMANGLED_len.__doc__,
            ),
        ),
        # abs(inval: Union[int, float, bool]) -> Union[int, float, bool]
        "abs": ScopeObject(
            name="abs",
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="abs",
                params=[
                    objpkg.Parameter(
                        name="inval",
                        value=None,
                        type_annotation="any",
                        type=ANY_TYPE,
                    ),
                ],
                self_type=ANY_TYPE,
                return_type=ANY_TYPE,
                doc=builtin.KMANGLED_abs.__doc__,
            ),
        ),
        # all_true(inval: Union[dict, list, schema, str]) -> bool
        "all_true": ScopeObject(
            name="all_true",
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="all_true",
                params=[
                    objpkg.Parameter(
                        name="inval",
                        value=None,
                        type_annotation="[]",
                        type=LIST_ANY_TYPE,
                    ),
                ],
                self_type=ANY_TYPE,
                return_type=BOOL_TYPE,
                doc=builtin.KMANGLED_all_true.__doc__,
            ),
        ),
        # any_true(inval: Union[dict, list, schema, str]) -> bool
        "any_true": ScopeObject(
            name="any_true",
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="any_true",
                params=[
                    objpkg.Parameter(
                        name="inval",
                        value=None,
                        type_annotation="[]",
                        type=LIST_ANY_TYPE,
                    ),
                ],
                self_type=ANY_TYPE,
                return_type=BOOL_TYPE,
                doc=builtin.KMANGLED_any_true.__doc__,
            ),
        ),
        # hex(number: int) -> str
        "hex": ScopeObject(
            name="hex",
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="hex",
                params=[
                    objpkg.Parameter(
                        name="number",
                        value=None,
                        type_annotation="int",
                        type=INT_TYPE,
                    ),
                ],
                self_type=ANY_TYPE,
                return_type=STR_TYPE,
                doc=builtin.KMANGLED_hex.__doc__,
            ),
        ),
        # bin(number: int) -> str
        "bin": ScopeObject(
            name="bin",
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="bin",
                params=[
                    objpkg.Parameter(
                        name="number",
                        value=None,
                        type_annotation="int",
                        type=INT_TYPE,
                    ),
                ],
                self_type=ANY_TYPE,
                return_type=STR_TYPE,
                doc=builtin.KMANGLED_bin.__doc__,
            ),
        ),
        # oct(number: int) -> str
        "oct": ScopeObject(
            name="oct",
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="oct",
                params=[
                    objpkg.Parameter(
                        name="number",
                        value=None,
                        type_annotation="int",
                        type=INT_TYPE,
                    ),
                ],
                self_type=ANY_TYPE,
                return_type=STR_TYPE,
                doc=builtin.KMANGLED_oct.__doc__,
            ),
        ),
        # ord(c: str) -> int
        "ord": ScopeObject(
            name="ord",
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="ord",
                params=[
                    objpkg.Parameter(
                        name="number",
                        value=None,
                        type_annotation="int",
                        type=STR_TYPE,
                    ),
                ],
                self_type=ANY_TYPE,
                return_type=INT_TYPE,
                doc=builtin.KMANGLED_ord.__doc__,
            ),
        ),
        # sorted(inval: Union[dict, list, schema, str]) -> List[Any]
        "sorted": ScopeObject(
            name="sorted",
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="sorted",
                params=[
                    objpkg.Parameter(
                        name="inval",
                        value=None,
                        type_annotation=ITERABLE_TYPE_STR,
                        type=ITERABLE_TYPE,
                    ),
                    objpkg.Parameter(
                        name="reverse",
                        value=objpkg.KCLFalseObject.instance(),
                        type_annotation="bool",
                        type=BOOL_TYPE,
                    ),
                ],
                self_type=ANY_TYPE,
                return_type=LIST_ANY_TYPE,
                doc=builtin.KMANGLED_sorted.__doc__,
            ),
        ),
        # range(start: int, stop: int, step: int = None) -> list
        "range": ScopeObject(
            name="range",
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="range",
                params=[
                    objpkg.Parameter(
                        name="start",
                        value=None,
                        type_annotation="int",
                        type=INT_TYPE,
                    ),
                    objpkg.Parameter(
                        name="stop",
                        value=None,
                        type_annotation="int",
                        type=INT_TYPE,
                    ),
                    objpkg.Parameter(
                        name="step",
                        value=objpkg.KCLNoneObject.instance(),
                        type_annotation="int",
                        type=INT_TYPE,
                    ),
                ],
                self_type=ANY_TYPE,
                return_type=objpkg.KCLListTypeObject(item_type=INT_TYPE),
                doc=builtin.KMANGLED_range.__doc__,
            ),
        ),
        # max(iterable: List[Any]) -> Any
        "max": ScopeObject(
            name="max",
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="max",
                params=[],
                self_type=ANY_TYPE,
                return_type=ANY_TYPE,
                doc=builtin.KMANGLED_max.__doc__,
            ),
        ),
        # min(iterable: List[Any]) -> Any
        "min": ScopeObject(
            name="min",
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="min",
                params=[],
                self_type=ANY_TYPE,
                return_type=ANY_TYPE,
                doc=builtin.KMANGLED_min.__doc__,
            ),
        ),
        # sum(iterable: List[Any], start: Any = 0) -> Any
        "sum": ScopeObject(
            name="sum",
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="sum",
                params=[
                    objpkg.Parameter(
                        name="iterable",
                        value=None,
                        type_annotation="[]",
                        type=LIST_ANY_TYPE,
                    ),
                    objpkg.Parameter(
                        name="start",
                        value=objpkg.KCLIntObject(0),
                        type_annotation="any",
                        type=ANY_TYPE,
                    ),
                ],
                self_type=ANY_TYPE,
                return_type=ANY_TYPE,
                doc=builtin.KMANGLED_sum.__doc__,
            ),
        ),
        # pow(x: number, y: number, z: number) -> number
        "pow": ScopeObject(
            name="pow",
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="pow",
                params=[
                    objpkg.Parameter(
                        name="x",
                        value=None,
                        type_annotation=NUMBER_TYPE_STR,
                        type=NUMBER_TYPE,
                    ),
                    objpkg.Parameter(
                        name="y",
                        value=None,
                        type_annotation=NUMBER_TYPE_STR,
                        type=NUMBER_TYPE,
                    ),
                    objpkg.Parameter(
                        name="z",
                        value=None,
                        type_annotation=NUMBER_TYPE_STR,
                        type=NUMBER_TYPE,
                    ),
                ],
                self_type=ANY_TYPE,
                return_type=NUMBER_TYPE,
                doc=builtin.KMANGLED_pow.__doc__,
            ),
        ),
        # round(number: number, ndigits: int) -> number
        "round": ScopeObject(
            name="round",
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="round",
                params=[
                    objpkg.Parameter(
                        name="number",
                        value=None,
                        type_annotation=NUMBER_TYPE_STR,
                        type=NUMBER_TYPE,
                    ),
                    objpkg.Parameter(
                        name="ndigits",
                        value=objpkg.KCLNoneObject.instance(),
                        type_annotation="int",
                        type=INT_TYPE,
                    ),
                ],
                self_type=ANY_TYPE,
                return_type=NUMBER_TYPE,
                doc=builtin.KMANGLED_round.__doc__,
            ),
        ),
        # zip(*args) -> List[Any]
        "zip": ScopeObject(
            name="zip",
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="zip",
                params=[],
                self_type=ANY_TYPE,
                return_type=LIST_ANY_TYPE,
                doc=builtin.KMANGLED_zip.__doc__,
                is_variadic=True,
            ),
        ),
        # int(number: number|str) -> int
        "int": ScopeObject(
            name="int",
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="int",
                params=[
                    objpkg.Parameter(
                        name="number",
                        value=None,
                        type_annotation="any",
                        type=ANY_TYPE,
                    ),
                    objpkg.Parameter(
                        name="base",
                        value=objpkg.KCLIntObject(10),
                        type_annotation="int",
                        type=ANY_TYPE,
                    ),
                ],
                self_type=ANY_TYPE,
                return_type=INT_TYPE,
                doc=builtin.KMANGLED_int.__doc__,
            ),
        ),
        # float(number: number|str) -> float
        "float": ScopeObject(
            name="float",
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="float",
                params=[
                    objpkg.Parameter(
                        name="number",
                        value=None,
                        type_annotation="any",
                        type=ANY_TYPE,
                    )
                ],
                self_type=ANY_TYPE,
                return_type=FLOAT_TYPE,
                doc=builtin.KMANGLED_float.__doc__,
            ),
        ),
        # list(x: any) -> []
        "list": ScopeObject(
            name="list",
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="list",
                params=[
                    objpkg.Parameter(
                        name="x",
                        value=None,
                        type_annotation="any",
                        type=ANY_TYPE,
                    )
                ],
                self_type=ANY_TYPE,
                return_type=LIST_ANY_TYPE,
                doc=builtin.KMANGLED_list.__doc__,
            ),
        ),
        # dict(x: any) -> {:}
        "dict": ScopeObject(
            name="dict",
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="dict",
                params=[
                    objpkg.Parameter(
                        name="x",
                        value=None,
                        type_annotation="any",
                        type=ANY_TYPE,
                    )
                ],
                self_type=ANY_TYPE,
                return_type=DICT_ANY_ANY_TYPE,
                doc=builtin.KMANGLED_dict.__doc__,
            ),
        ),
        # bool(x: any) -> bool
        "bool": ScopeObject(
            name="bool",
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="bool",
                params=[
                    objpkg.Parameter(
                        name="x",
                        value=None,
                        type_annotation="any",
                        type=ANY_TYPE,
                    )
                ],
                self_type=ANY_TYPE,
                return_type=BOOL_TYPE,
                doc=builtin.KMANGLED_bool.__doc__,
            ),
        ),
        # str(obj: any) -> str
        "str": ScopeObject(
            name="str",
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="str",
                params=[
                    objpkg.Parameter(
                        name="x",
                        value=None,
                        type_annotation="any",
                        type=ANY_TYPE,
                    )
                ],
                self_type=ANY_TYPE,
                return_type=STR_TYPE,
                doc=builtin.KMANGLED_str.__doc__,
            ),
        ),
        # typeof(x: any, *, full_name: bool = False) -> str:
        "typeof": ScopeObject(
            name="typeof",
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="typeof",
                params=[
                    objpkg.Parameter(
                        name="x",
                        value=None,
                        type_annotation="any",
                        type=ANY_TYPE,
                    ),
                    objpkg.Parameter(
                        name="full_name",
                        value=objpkg.KCLFalseObject.instance(),
                        type_annotation="bool",
                        type=BOOL_TYPE,
                    ),
                ],
                self_type=ANY_TYPE,
                return_type=STR_TYPE,
                doc=builtin.KMANGLED_typeof.__doc__,
            ),
        ),
    }
)

# System module types table
MODULE_SCOPE_MAPPING: Dict[str, Scope] = {
    **{name: Scope(elems={}) for name in builtin.STANDARD_SYSTEM_MODULES},
    **{
        "units": Scope(
            elems={
                "NumberMultiplier": ScopeObject(
                    name="NumberMultiplier",
                    node=None,
                    type=objpkg.KCLNumberMultiplierTypeObject(),
                ),
            }
        ),
    },
}

try:
    # Plugin module types table
    PLUGIN_SCOPE_MAPPING: Dict[str, Scope] = {
        "{}".format(plugin.PLUGIN_MODULE_NAME + name): Scope(elems={})
        for name in plugin.get_plugin_names()
    }
except Exception:
    PLUGIN_SCOPE_MAPPING: Dict[str, Scope] = {}


# Member function or property types table e.g., str.replace and schema.instances
SCHEMA_TYPE_MEMBER_SCOPE: Scope = Scope(
    elems={
        "instances": ScopeObject(
            name=objpkg.KCLSchemaTypeObject.instances.__name__,
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name=objpkg.KCLSchemaTypeObject.instances.__name__,
                params=[],
                self_type=DICT_ANY_ANY_TYPE,
                return_type=LIST_ANY_TYPE,
                doc=objpkg.KCLSchemaTypeObject.instances.__doc__,
            ),
        )
    }
)

STR_TYPE_MEMBER_SCOPE: Scope = Scope(
    elems={
        "capitalize": ScopeObject(
            name="".capitalize.__name__,
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="".capitalize.__name__,
                params=[],
                self_type=STR_TYPE,
                return_type=STR_TYPE,
                doc="".capitalize.__doc__,
            ),
        ),
        "count": ScopeObject(
            name="".count.__name__,
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="".count.__name__,
                params=[],
                self_type=STR_TYPE,
                return_type=INT_TYPE,
                doc="".count.__doc__,
            ),
        ),
        "endswith": ScopeObject(
            name="".endswith.__name__,
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="".endswith.__name__,
                params=[],
                self_type=STR_TYPE,
                return_type=BOOL_TYPE,
                doc="".endswith.__doc__,
            ),
        ),
        "find": ScopeObject(
            name="".find.__name__,
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="".find.__name__,
                params=[],
                self_type=STR_TYPE,
                return_type=INT_TYPE,
                doc="".find.__doc__,
            ),
        ),
        "format": ScopeObject(
            name="".format.__name__,
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="".format.__name__,
                params=[],
                self_type=STR_TYPE,
                return_type=STR_TYPE,
                doc="".format.__doc__,
            ),
        ),
        "index": ScopeObject(
            name="".index.__name__,
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="".index.__name__,
                params=[],
                self_type=STR_TYPE,
                return_type=INT_TYPE,
                doc="".index.__doc__,
            ),
        ),
        "isalnum": ScopeObject(
            name="".isalnum.__name__,
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="".isalnum.__name__,
                params=[],
                self_type=STR_TYPE,
                return_type=BOOL_TYPE,
                doc="".isalnum.__doc__,
            ),
        ),
        "isalpha": ScopeObject(
            name="".isalpha.__name__,
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="".isalpha.__name__,
                params=[],
                self_type=STR_TYPE,
                return_type=BOOL_TYPE,
                doc="".isalpha.__doc__,
            ),
        ),
        "isdigit": ScopeObject(
            name="".isdigit.__name__,
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="".isdigit.__name__,
                params=[],
                self_type=STR_TYPE,
                return_type=BOOL_TYPE,
                doc="".isdigit.__doc__,
            ),
        ),
        "islower": ScopeObject(
            name="".islower.__name__,
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="".islower.__name__,
                params=[],
                self_type=STR_TYPE,
                return_type=BOOL_TYPE,
                doc="".islower.__doc__,
            ),
        ),
        "isspace": ScopeObject(
            name="".isspace.__name__,
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="".isspace.__name__,
                params=[],
                self_type=STR_TYPE,
                return_type=BOOL_TYPE,
                doc="".isspace.__doc__,
            ),
        ),
        "istitle": ScopeObject(
            name="".istitle.__name__,
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="".istitle.__name__,
                params=[],
                self_type=STR_TYPE,
                return_type=BOOL_TYPE,
                doc="".istitle.__doc__,
            ),
        ),
        "isupper": ScopeObject(
            name="".isupper.__name__,
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="".isupper.__name__,
                params=[],
                self_type=STR_TYPE,
                return_type=BOOL_TYPE,
                doc="".isupper.__doc__,
            ),
        ),
        "join": ScopeObject(
            name="".join.__name__,
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="".join.__name__,
                params=[],
                self_type=STR_TYPE,
                return_type=STR_TYPE,
                doc="".join.__doc__,
            ),
        ),
        "lower": ScopeObject(
            name="".lower.__name__,
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="".lower.__name__,
                params=[],
                self_type=STR_TYPE,
                return_type=STR_TYPE,
                doc="".lower.__doc__,
            ),
        ),
        "upper": ScopeObject(
            name="".upper.__name__,
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="".upper.__name__,
                params=[],
                self_type=STR_TYPE,
                return_type=STR_TYPE,
                doc="".upper.__doc__,
            ),
        ),
        "lstrip": ScopeObject(
            name="".lstrip.__name__,
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="".lstrip.__name__,
                params=[],
                self_type=STR_TYPE,
                return_type=STR_TYPE,
                doc="".lstrip.__doc__,
            ),
        ),
        "rstrip": ScopeObject(
            name="".rstrip.__name__,
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="".rstrip.__name__,
                params=[],
                self_type=STR_TYPE,
                return_type=STR_TYPE,
                doc="".rstrip.__doc__,
            ),
        ),
        "replace": ScopeObject(
            name="".replace.__name__,
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="".replace.__name__,
                params=[],
                self_type=STR_TYPE,
                return_type=STR_TYPE,
                doc="".replace.__doc__,
            ),
        ),
        "rfind": ScopeObject(
            name="".rfind.__name__,
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="".rfind.__name__,
                params=[],
                self_type=STR_TYPE,
                return_type=INT_TYPE,
                doc="".rfind.__doc__,
            ),
        ),
        "rindex": ScopeObject(
            name="".rindex.__name__,
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="".rindex.__name__,
                params=[],
                self_type=STR_TYPE,
                return_type=INT_TYPE,
                doc="".rindex.__doc__,
            ),
        ),
        "rsplit": ScopeObject(
            name="".rsplit.__name__,
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="".rsplit.__name__,
                params=[],
                self_type=STR_TYPE,
                return_type=LIST_STR_TYPE,
                doc="".rsplit.__doc__,
            ),
        ),
        "split": ScopeObject(
            name="".split.__name__,
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="".split.__name__,
                params=[],
                self_type=STR_TYPE,
                return_type=LIST_STR_TYPE,
                doc="".split.__doc__,
            ),
        ),
        "splitlines": ScopeObject(
            name="".splitlines.__name__,
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="".splitlines.__name__,
                params=[],
                self_type=STR_TYPE,
                return_type=LIST_STR_TYPE,
                doc="".splitlines.__doc__,
            ),
        ),
        "startswith": ScopeObject(
            name="".startswith.__name__,
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="".startswith.__name__,
                params=[],
                self_type=STR_TYPE,
                return_type=BOOL_TYPE,
                doc="".startswith.__doc__,
            ),
        ),
        "strip": ScopeObject(
            name="".strip.__name__,
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="".strip.__name__,
                params=[],
                self_type=STR_TYPE,
                return_type=STR_TYPE,
                doc="".strip.__doc__,
            ),
        ),
        "title": ScopeObject(
            name="".title.__name__,
            node=None,
            type=objpkg.KCLFunctionTypeObject(
                name="".title.__name__,
                params=[],
                self_type=STR_TYPE,
                return_type=STR_TYPE,
                doc="".title.__doc__,
            ),
        ),
    }
)


@dataclass
class ProgramScope:
    scope_map: Dict[str, Scope]
    builtin_scope: Scope = field(default_factory=lambda: BUILTIN_SCOPE)
    plugin_scope_map: Scope = field(default_factory=lambda: PLUGIN_SCOPE_MAPPING)
    system_module_scope_map: Dict[str, Scope] = field(
        default_factory=lambda: MODULE_SCOPE_MAPPING
    )
    member_scope_map: Dict[str, Scope] = field(
        default_factory=lambda: {
            objpkg.KCLTypeKind.StrKind: SCHEMA_TYPE_MEMBER_SCOPE,
            objpkg.KCLTypeKind.SchemaKind: STR_TYPE_MEMBER_SCOPE,
        }
    )
    schema_reference: objpkg.SchemaTypeRefGraph = field(default_factory=lambda: None)

    @property
    def main_scope(self) -> Scope:
        return self.scope_map[ast.Program.MAIN_PKGPATH]

    @property
    def pkgpaths(self) -> List[str]:
        return list(self.scope_map.keys())
