"""The `checker` file mainly contains the function `ResolveProgram`
based on the AST walker, which is used to run semantic checking and
type checking of KCL.

:copyright: Copyright 2020 The KCL Authors. All rights reserved.
"""

import os
import pathlib
import typing
from enum import IntEnum
from typing import cast, Union, List, Dict, Tuple, Optional
from dataclasses import dataclass

import kclvm.api.object as objpkg
import kclvm.api.object.internal as internal
import kclvm.kcl.error as kcl_error
import kclvm.kcl.info as kcl_info
import kclvm.kcl.ast as ast
import kclvm.compiler.astutil as astutil
import kclvm.compiler.extension.builtin as builtin
import kclvm.compiler.extension.plugin as plugin

from kclvm.compiler.build.utils import units
from kclvm.kcl.types.scope import (
    Scope,
    PackageScope,
    ScopeObject,
    ProgramScope,
    BUILTIN_SCOPE,
    DECORATOR_SCOPE,
    PLUGIN_SCOPE_MAPPING,
    MODULE_SCOPE_MAPPING,
    SCHEMA_TYPE_MEMBER_SCOPE,
    STR_TYPE_MEMBER_SCOPE,
)
from kclvm.kcl.types.type import (
    Type,
    ANY_TYPE,
    VOID_TYPE,
    NONE_TYPE,
    STR_TYPE,
    BOOL_TYPE,
    INT_TYPE,
    TRUE_LIT_TYPE,
    FALSE_LIT_TYPE,
    DICT_STR_ANY_TYPE,
    RESERVED_TYPE_IDENTIFIERS,
    KEY_KINDS,
    sup,
    assignable_to,
    is_upper_bound,
    infer_to_variable_type,
    is_kind_type_or_kind_union_type,
    type_to_kcl_type_annotation_str,
)
from kclvm.kcl.types.type_parser import parse_type_str
from kclvm.kcl.types.calculation import binary, compare, unary
from kclvm.kcl.types.walker import WalkType
from kclvm.internal.util import check_utils

ITER_TYPES = (
    objpkg.KCLAnyTypeObject,
    objpkg.KCLListTypeObject,
    objpkg.KCLStringTypeObject,
    objpkg.KCLStringLitTypeObject,
    objpkg.KCLDictTypeObject,
    objpkg.KCLSchemaTypeObject,
)
MAX_SCOPE_SCAN_COUNT = 3
VALID_FORMAT_SPEC_SET = {"#json", "#yaml"}
TYPE_KIND_BUILTIN_FUNCTION_MAPPING = {
    objpkg.KCLTypeKind.BoolKind: "bool",
    objpkg.KCLTypeKind.IntKind: "int",
    objpkg.KCLTypeKind.FloatKind: "float",
    objpkg.KCLTypeKind.StrKind: "str",
    objpkg.KCLTypeKind.ListKind: "list",
    objpkg.KCLTypeKind.DictKind: "dict",
}


class SwitchConfigExprContextState(IntEnum):
    SWITCH_CONFIG_EXPR_ONCE = 1
    KEEP_CONFIG_EXPR_UNCHANGED = 0


@dataclass
class CheckConfig:
    raise_err: bool = True
    config_attr_auto_fix: bool = False


class BaseTypeChecker(ast.TreeWalker):
    def __init__(self, program: ast.Program, config: CheckConfig = CheckConfig()):
        # The type checker config
        self.config: CheckConfig = config
        # The AST program reference
        self.program: ast.Program = program
        # Current package path, default is the main package path
        self.pkgpath = ast.Program.MAIN_PKGPATH
        # Current filename
        self.filename = ""
        # The scope mapping between `pkgpath` and `scope`
        self.scope_map: Dict[str, Scope] = {}
        # Current scope
        self.scope: Optional[Scope] = None
        # Current schema type
        self.in_schema_type: Optional[Type] = None
        # Current schema expr type stack
        self.config_expr_context: List[Optional[ScopeObject]] = []
        # Check error list
        self.errs: List[kcl_error.KCLException] = []
        # Local vars
        self._local_vars: List[str] = []
        # Schema type reference graph
        self.schema_reference: objpkg.SchemaTypeRefGraph = objpkg.SchemaTypeRefGraph()
        # Schema types mapping
        self.schema_mapping: Dict[str, Type] = {}
        # Package path import reference graph
        self.import_reference: objpkg.RefGraph = objpkg.RefGraph()
        # In lambda expression level
        self._is_in_lambda_expr: List[bool] = [False]
        # Reset scope status
        self.reset_scopes()
        # Set __main_ package context
        self.change_package_context(self.pkgpath, self.filename)

    @staticmethod
    def reset_scopes():
        BUILTIN_SCOPE.children = []

    def new_config_expr_context_item(
        self,
        name: str = None,
        type_obj: Optional[Type] = None,
        node: Optional[ast.AST] = None,
    ) -> ScopeObject:
        return ScopeObject(
            name=name,
            node=node,
            type=type_obj,
            pos=ast.Position(filename=node.filename, line=node.line, column=node.column)
            if node
            else None,
            end=ast.Position(
                filename=node.filename, line=node.end_line, column=node.end_column
            )
            if node
            else None,
        )

    def find_schema_attr_obj_from_schema_expr_stack(
        self, key_name: str
    ) -> Optional[ScopeObject]:
        """Finds the items needed to switch the context by name 'key_name'

            At present, only when the top item of the stack is 'KCLSchemaTypeObject' or 'KCLDictTypeObject',
            it will return the next item (the attribute named 'key_name' in 'KCLSchemaTypeObject'
            or the value of 'key_name' in 'KCLDictTypeObject') needed to be pushed.
            If the top item of the stack is not 'KCLSchemaTypeObject' or 'KCLDictTypeObject',
            it will return 'None'.

        Args:
            key_name: The name of the item needed to be pushed onto the 'config_expr_context' stack

        Returns:
            The item needed to be pushed onto the 'config_expr_context' stack

        """
        if (
            not key_name
            or not self.config_expr_context
            or not self.config_expr_context[-1]
        ):
            return None
        if not isinstance(self.config_expr_context[-1], ScopeObject):
            check_utils.alert_internal_bug()

        if isinstance(self.config_expr_context[-1].type, objpkg.KCLSchemaTypeObject):
            schema_type = typing.cast(
                objpkg.KCLSchemaTypeObject, self.config_expr_context[-1].type
            )
            attr_type_obj = schema_type.get_obj_of_attr(key_name)
            if not attr_type_obj and schema_type.index_signature:
                ctx_obj = self.new_config_expr_context_item(
                    name=key_name,
                    type_obj=schema_type.index_signature.value_kcl_type,
                    node=schema_type.index_signature.node,
                )
            elif attr_type_obj:
                ctx_obj = self.new_config_expr_context_item(
                    name=key_name,
                    type_obj=attr_type_obj.attr_type,
                    node=attr_type_obj.attr_node,
                )
            else:
                return None
            return ctx_obj
        if isinstance(self.config_expr_context[-1].type, objpkg.KCLDictTypeObject):
            dict_type = typing.cast(
                objpkg.KCLDictTypeObject, self.config_expr_context[-1].type
            )
            ctx_obj = self.new_config_expr_context_item(
                name=key_name,
                type_obj=dict_type.value_type,
                node=self.config_expr_context[-1].node,
            )
            return ctx_obj
        return None

    def switch_config_expr_context_by_key(self, key: ast.Expr) -> int:
        """Switch the context in 'config_expr_context' stack by AST nodes 'Identifier', 'Subscript' or 'Literal'

        Args:
            key: AST nodes 'Identifier', 'Subscript' or 'Literal'

        Returns:
            push stack times

        """
        names = []
        if not key:
            return SwitchConfigExprContextState.KEEP_CONFIG_EXPR_UNCHANGED
        if isinstance(key, ast.Identifier):
            names = key.names
        elif isinstance(key, ast.Subscript):
            if isinstance(key.value, ast.Identifier) and isinstance(
                key.index, ast.NumberLit
            ):
                names = key.value.names
        elif isinstance(key, ast.Literal):
            names = [key.value]
        else:
            return SwitchConfigExprContextState.KEEP_CONFIG_EXPR_UNCHANGED
        return self.switch_config_expr_context_by_names(names)

    def switch_config_expr_context_by_names(
        self, names: List[Union[str, float, int]]
    ) -> int:
        """Switch the context in 'config_expr_context' stack by names

            Traverse all name in 'names', find the next item that needs to be pushed into the stack,
            according to name and the top context of the stack, and push the item into the stack.

        Args:
            names: A list of string containing the names of items to be pushed

        Returns:
            push stack times

        """
        stack_depth = 0
        for name in names:
            stack_depth += self.switch_config_expr_context_by_name(name)
        return stack_depth

    def switch_config_expr_context_by_name(self, name: str) -> int:
        """Switch the context in 'config_expr_context' stack by name

            find the next item that needs to be pushed into the stack,
            according to name and the top context of the stack, and push the item into the stack.

        Args:
            name: the name of item to be pushed

        Returns:
            push stack times

        """
        ctx_obj = self.find_schema_attr_obj_from_schema_expr_stack(name)
        return self.switch_config_expr_context(ctx_obj)

    def switch_config_expr_context(
        self, config_ctx_obj: ScopeObject
    ) -> SwitchConfigExprContextState:
        """Push method for the 'config_expr_context' stack

        Args:
            config_ctx_obj: the item needed to be pushed

        Returns:
            push stack times

        """
        self.config_expr_context.append(config_ctx_obj)
        return SwitchConfigExprContextState.SWITCH_CONFIG_EXPR_ONCE

    def restore_config_expr_context(self) -> Optional[objpkg.KCLSchemaTypeObject]:
        """Pop method for the 'config_expr_context' stack

        Returns:
            the item poped from stack

        """
        return self.config_expr_context.pop(-1) if self.config_expr_context else None

    def clear_config_expr_context(self, stack_depth: int = 0, clear_all: bool = False):
        """Pop_All method for the 'config_expr_context' stack

        Args:
            stack_depth: 'stack_depth' is the number of stacks that need to be popped
            clear_all: 'clear_all' is True to clear all the items of the stack

        """
        if clear_all:
            self.config_expr_context.clear()
        else:
            while stack_depth > 0:
                stack_depth -= 1
                self.restore_config_expr_context()

    def check_config_expr_by_key_name(
        self, name: str, key: ast.AST, check_rules: List[typing.Callable]
    ):
        """Check whether the key of config expr meets the constraints of schema attributes such as final, defined.

        Args:
            name: the name of key
            key: the ast node of key
            check_rules: the constraints, such as 'check_defined'

        """
        if name and self.config_expr_context and self.config_expr_context[-1]:
            self.check_attr(
                attr=name,
                node=key,
                obj=self.config_expr_context[-1].type,
                check_rules=check_rules,
            )

    def check_config_entry(
        self, key: ast.Expr, value: ast.Expr, check_rules: List[typing.Callable]
    ) -> typing.Optional[ast.Expr]:
        """Check the key-value in 'ConfigExpr', such as check_defined and check_type

        Notes:
            If the top item of the 'config_expr_context' stack is 'None', the check will be skipped.

        Args:
            key: the key of 'ConfigExpr'.
            value: the value of 'ConfigExpr'.
            check_rules: Some checks on the key individually，such as check_defined.

        """
        if not (key and self.config_expr_context and self.config_expr_context[-1]):
            return
        names: List[Union[str, float, int]] = []
        has_index = False

        def _check() -> typing.Optional[ast.Expr]:
            stack_depth = 0
            fix_call_expr = None
            for name in names:
                self.check_config_expr_by_key_name(name, key, check_rules)
                stack_depth += self.switch_config_expr_context_by_name(name)
            value_tpe = self.expr(value)
            if len(names) > 1:
                for _ in range(len(names) - 1):
                    value_tpe = objpkg.KCLDictTypeObject(
                        key_type=STR_TYPE, value_type=value_tpe
                    )
            if has_index:
                value_tpe = (
                    objpkg.KCLListTypeObject(value_tpe)
                    if value_tpe and isinstance(value_tpe, objpkg.KCLBaseTypeObject)
                    else None
                )
            if self.config.config_attr_auto_fix:
                try:
                    _check_type(value_tpe)
                # Type check error and fix the attr type with the builtin functions
                except Exception:
                    expected_attr_type = self.config_expr_context[-1].type.type_kind()
                    if expected_attr_type in TYPE_KIND_BUILTIN_FUNCTION_MAPPING:
                        func_name = TYPE_KIND_BUILTIN_FUNCTION_MAPPING[
                            expected_attr_type
                        ]
                        fix_call_expr = ast.CallExpr(line=key.line, column=key.column)
                        fix_call_expr.func = ast.ASTFactory.get_ast_identifier(
                            func_name
                        )
                        fix_call_expr.args = [value]
            else:
                _check_type(value_tpe)
            self.clear_config_expr_context(stack_depth=stack_depth)
            return fix_call_expr

        def _check_type(value_tpe: Type):
            if value_tpe and self.config_expr_context and self.config_expr_context[-1]:
                self.must_assignable_to(
                    node=key,
                    tpe=value_tpe or ANY_TYPE,
                    expected_type=self.config_expr_context[-1].type,
                    expected_node=self.config_expr_context[-1].node,
                )

        if isinstance(key, ast.Identifier):
            names = key.names
        elif isinstance(key, ast.Subscript):
            if isinstance(key.value, ast.Identifier) and isinstance(
                key.index, ast.NumberLit
            ):
                names = key.value.names
                has_index = True
            else:
                return
        elif isinstance(key, ast.Literal):
            names = [key.value]
        else:
            return
        return _check()

    def get_node_name(self, t: ast.AST):
        """Get the ast.AST node name"""
        assert isinstance(t, ast.AST), str(type(t))
        return t.type

    def generic_walk(self, t: ast.AST):
        """Called if no explicit walker function exists for a node."""
        raise Exception(
            f"The function walk_{t.type} is not defined in the type checker."
        )

    def raise_err(
        self,
        nodes: [ast.AST],
        category: kcl_error.ErrType = kcl_error.ErrType.CompileError_TYPE,
        msg: str = "",
        file_msgs=None,
        file_levels=None,
    ):
        """Raise a KCL compile error"""
        err = kcl_error.get_exception(
            err_type=category,
            file_msgs=[
                kcl_error.ErrFileMsg(
                    filename=node.filename,
                    line_no=node.line,
                    col_no=node.column,
                    arg_msg=(file_msgs[i] if i < len(file_msgs) else file_msgs[-1])
                    if file_msgs
                    else None,
                    err_level=(
                        file_levels[i] if i < len(file_levels) else file_levels[-1]
                    )
                    if file_levels
                    else None,
                )
                for i, node in enumerate(nodes)
                if node
            ],
            arg_msg=msg,
        )
        if self.config.raise_err:
            raise err
        self.errs.append(err)

    def change_package_context(self, pkgpath: str, filename: str):
        """Change the package scope context with pkgpath and filename"""
        if not pkgpath:
            return
        if pkgpath not in self.scope_map:
            self.scope_map[pkgpath] = PackageScope(
                parent=BUILTIN_SCOPE,
                file_begin_position_map={
                    module.filename: ast.Position(
                        filename=module.filename,
                        line=module.line,
                        column=module.column,
                    )
                    for module in self.program.pkgs[pkgpath]
                },
                file_end_position_map={
                    module.filename: ast.Position(
                        filename=module.filename,
                        line=module.end_line,
                        column=module.end_column,
                    )
                    for module in self.program.pkgs[pkgpath]
                },
            )
            BUILTIN_SCOPE.children.append(self.scope_map[pkgpath])
        self.pkgpath = pkgpath
        self.filename = filename
        self.scope = self.scope_map[pkgpath]

    def check(self, pkgpath: str = ast.Program.MAIN_PKGPATH) -> ProgramScope:
        """The check main function"""
        self.check_import(pkgpath)
        self.init_global_types()
        for module in self.program.pkgs[pkgpath]:
            self.filename = module.filename
            self.walk(module)
        self.scope_map[pkgpath] = self.scope
        return ProgramScope(
            scope_map=self.scope_map,
            schema_reference=self.schema_reference,
        )

    def check_import(self, pkgpath: str = ast.Program.MAIN_PKGPATH):
        """The import check function"""
        self.pkgpath = pkgpath
        self.change_package_context(pkgpath, self.filename)
        self.init_import_list()

    def build_rule_protocol_type(
        self, t: ast.RuleStmt
    ) -> Optional[objpkg.KCLSchemaDefTypeObject]:
        if t.for_host_name and isinstance(t.for_host_name, ast.Identifier):
            if len(t.for_host_name.names) > 2:
                self.raise_err(
                    category=kcl_error.ErrType.MultiInheritError_TYPE,
                    nodes=[t.for_host_name],
                    msg=kcl_error.MULTI_INHERIT_MSG.format(t.name),
                )
                return None
            tpe = self.expr(t.for_host_name)
            if not isinstance(tpe, objpkg.KCLSchemaDefTypeObject):
                self.raise_err(
                    category=kcl_error.ErrType.IllegalInheritError_TYPE,
                    nodes=[t],
                    msg=f"invalid schema inherit object type '{tpe.type_str()}'",
                )
                return None
            return cast(objpkg.KCLSchemaDefTypeObject, tpe)
        return None

    def build_schema_protocol_type(
        self, t: ast.SchemaStmt
    ) -> Optional[objpkg.KCLSchemaDefTypeObject]:
        # Mixin type check with protocol
        if not t.is_mixin and t.for_host_name:
            self.raise_err(
                category=kcl_error.ErrType.IllegalInheritError_TYPE,
                nodes=[t.for_host_name],
                msg="only schema mixin can inherit from protocol",
            )
            return None
        if t.is_mixin and t.for_host_name:
            if len(t.for_host_name.names) > 2:
                self.raise_err(
                    category=kcl_error.ErrType.MultiInheritError_TYPE,
                    nodes=[t.for_host_name],
                    msg=kcl_error.MULTI_INHERIT_MSG.format(t.name),
                )
                return None
            tpe = self.expr(t.for_host_name)
            if not isinstance(tpe, objpkg.KCLSchemaDefTypeObject):
                self.raise_err(
                    category=kcl_error.ErrType.IllegalInheritError_TYPE,
                    nodes=[t],
                    msg=f"invalid schema inherit object type '{tpe.type_str()}'",
                )
                return None
            return cast(objpkg.KCLSchemaDefTypeObject, tpe)
        return None

    def build_schema_parent_type(
        self, t: ast.SchemaStmt
    ) -> Optional[objpkg.KCLSchemaDefTypeObject]:
        if t.parent_name:
            if len(t.parent_name.names) > 2:
                self.raise_err(
                    category=kcl_error.ErrType.MultiInheritError_TYPE,
                    nodes=[t.parent_name],
                    msg=kcl_error.MULTI_INHERIT_MSG.format(t.name),
                )
                return None
            schema_parent_type = self.expr(t.parent_name)
            if not isinstance(schema_parent_type, objpkg.KCLSchemaDefTypeObject):
                self.raise_err(
                    category=kcl_error.ErrType.IllegalInheritError_TYPE,
                    nodes=[t],
                    msg=f"illegal schema inherit object type '{schema_parent_type.type_str()}'",
                )
                return None
            return schema_parent_type
        return None

    def build_schema_type(
        self,
        t: ast.SchemaStmt,
        base_def: objpkg.KCLSchemaDefTypeObject = None,
        protocol_def: objpkg.KCLSchemaDefTypeObject = None,
        should_add_schema_ref: bool = False,
    ) -> objpkg.KCLSchemaDefTypeObject:
        """Build a schema type and check"""
        # Base schema: get the parent type obj of the schema if exist
        if t.name in RESERVED_TYPE_IDENTIFIERS:
            self.raise_err(
                [t],
                kcl_error.ErrType.IllegalInheritError_TYPE,
                "schema name '{}' cannot be the same as the built-in types ({})".format(
                    t.name, ", ".join(RESERVED_TYPE_IDENTIFIERS)
                ),
            )
        if t.is_protocol and not t.has_only_attribute_definitions():
            self.raise_err(
                [t],
                kcl_error.ErrType.CompileError_TYPE,
                msg="a protocol is only allowed to define attributes in it",
            )
        base = base_def.schema_type if base_def else None
        protocol = protocol_def.schema_type if protocol_def else None
        parent_name_str = t.parent_name.get_name() if t.parent_name else ""
        if parent_name_str.endswith("Mixin"):
            self.raise_err(
                [t],
                kcl_error.ErrType.IllegalInheritError_TYPE,
                f"mixin inheritance {parent_name_str} is prohibited",
            )
        schema_attr_names = t.GetLeftIdentifierList()
        # Index signature
        index_sign_name = t.GetIndexSignatureAttrName()
        index_sign_obj = None
        if index_sign_name and index_sign_name in schema_attr_names:
            self.raise_err(
                nodes=[t.index_signature],
                category=kcl_error.ErrType.IndexSignatureError_TYPE,
                msg=f"index signature attribute name '{index_sign_name}' "
                "cannot have the same name as schema attributes",
            )
        if t.index_signature:
            key_kcl_type = self.parse_type_str_with_scope(
                t.index_signature.key_type, t.index_signature
            )
            if not is_kind_type_or_kind_union_type(key_kcl_type, KEY_KINDS):
                self.raise_err(
                    nodes=[t.index_signature],
                    category=kcl_error.ErrType.IndexSignatureError_TYPE,
                    msg=f"invalid index signature key type: '{key_kcl_type.type_str()}'",
                )
            value_kcl_type = self.parse_type_str_with_scope(
                t.index_signature.value_type, t.index_signature
            )
            index_sign_obj = objpkg.KCLSchemaIndexSignatureObject(
                key_name=t.index_signature.key_name,
                key_type=t.index_signature.key_type,
                value_type=t.index_signature.value_type,
                any_other=t.index_signature.any_other,
                key_kcl_type=key_kcl_type,
                value_kcl_type=value_kcl_type,
                node=t.index_signature,
            )
            t.index_signature.key_type = type_to_kcl_type_annotation_str(key_kcl_type)
            t.index_signature.value_type = type_to_kcl_type_annotation_str(
                value_kcl_type
            )

        # Schema attr type map
        attr_obj_map = {
            objpkg.SCHEMA_SETTINGS_ATTR_NAME: objpkg.KCLSchemaAttrObject(
                attr_type=DICT_STR_ANY_TYPE
            )
        }
        for attr in t.GetAttrList():
            name = (
                attr.name
                if isinstance(attr, ast.SchemaAttr)
                else attr.target.get_first_name()
            )
            if isinstance(attr, ast.SchemaAttr):
                tpe = self.parse_type_str_with_scope(attr.type_str, attr)
                attr.type_str = type_to_kcl_type_annotation_str(tpe)
            else:
                tpe = self.parse_type_str_with_scope(attr.value.name.get_name(), attr)
                tpe_str = type_to_kcl_type_annotation_str(tpe)
                names = (
                    tpe_str.rsplit(".", 1)
                    if tpe_str.startswith("@")
                    else tpe_str.split(".")
                )
                attr.value.name.names = names
            base_tpe = (base.get_type_of_attr(name) if base else None) or ANY_TYPE
            if name not in attr_obj_map:
                existed_attr = base.get_obj_of_attr(name) if base else None
                attr_obj_map[name] = objpkg.KCLSchemaAttrObject(
                    is_optional=existed_attr.is_optional
                    if existed_attr
                    else isinstance(attr, ast.SchemaAttr) and attr.is_optional,
                    is_final=False,
                    has_default=(
                        (isinstance(attr, ast.SchemaAttr) and attr.value is not None)
                        or (existed_attr and existed_attr.has_default)
                    ),
                    attr_type=tpe,
                    attr_node=attr,
                )
            if not is_upper_bound(
                attr_obj_map[name].attr_type, tpe
            ) or not is_upper_bound(base_tpe, tpe):
                self.raise_err(
                    [attr],
                    kcl_error.ErrType.TypeError_Compile_TYPE,
                    f"can't change schema field type of '{name}'",
                )
            if (
                isinstance(attr, ast.SchemaAttr)
                and attr.is_optional
                and not attr_obj_map[name].is_optional
            ):
                self.raise_err(
                    [attr],
                    msg=f"can't change the required schema attribute of '{name}' to optional",
                )
            if (
                index_sign_obj
                and not index_sign_obj.any_other
                and not is_upper_bound(index_sign_obj.value_kcl_type, tpe)
            ):
                self.raise_err(
                    nodes=[attr],
                    category=kcl_error.ErrType.IndexSignatureError_TYPE,
                    msg=f"the type '{tpe.type_str()}' of schema attribute '{name}' "
                    f"does not meet the index signature definition {index_sign_obj.def_str()}",
                )

        mixin_name_list = []
        for mixin in t.mixins or []:
            mixin_names = mixin.names
            if mixin.pkgpath:
                mixin_names[0] = f"@{mixin.pkgpath}"
            if not mixin_names[-1].endswith("Mixin"):
                self.raise_err(
                    [mixin],
                    kcl_error.ErrType.MixinNamingError_TYPE,
                    f"a valid mixin name should end with 'Mixin', got '{mixin_names[-1]}'",
                )
            mixin_name_list.append(mixin.get_name())
            mixin_type = self.expr(mixin)
            if mixin_type == ANY_TYPE:
                continue
            if not isinstance(mixin_type, objpkg.KCLSchemaDefTypeObject):
                self.raise_err(
                    [mixin],
                    kcl_error.ErrType.CompileError_TYPE,
                    msg=f"illegal schema mixin object type '{mixin_type.type_str()}'",
                )
            else:
                for name, attr_obj in mixin_type.schema_type.attr_obj_map.items():
                    if name not in attr_obj_map:
                        attr_obj_map[name] = attr_obj

        params: List[objpkg.Parameter] = []
        # Schema arguments
        if t.args:
            for i, arg in enumerate(t.args.args):
                name = arg.get_name()
                if name in schema_attr_names:
                    self.raise_err(
                        [arg],
                        msg=f"Unexpected parameter name '{name}' "
                        "with the same name as the schema attribute",
                    )
                type_annotation = t.args.GetArgType(i)
                type_node = self.parse_type_str_with_scope(
                    type_annotation, t.args.args[i]
                )
                default = t.args.GetArgDefault(i)
                params.append(
                    objpkg.Parameter(
                        name=name,
                        value=objpkg.to_kcl_obj(default),
                        type_annotation=type_annotation,
                        type=type_node,
                    )
                )
                t.args.SetArgType(i, type_to_kcl_type_annotation_str(type_node))
        runtime_type = objpkg.KCLSchemaTypeObject.schema_runtime_type(
            t.name, self.pkgpath
        )
        if should_add_schema_ref and self.schema_reference.add_node_judge_cycle(
            runtime_type, base.runtime_type if base else ""
        ):
            base_name = base.name if base else ""
            self.raise_err(
                [t],
                kcl_error.ErrType.CycleInheritError_TYPE,
                f"{t.name} and {base_name}",
            )
        schema_type = objpkg.KCLSchemaTypeObject(
            name=t.name,
            is_mixin=t.is_mixin,
            is_protocol=t.is_protocol,
            pkgpath=self.pkgpath,
            filename=self.filename,
            doc=t.doc,
            base=base,
            protocol=protocol,
            runtime_type=runtime_type,
            mixins_names=mixin_name_list,
            attr_list=schema_attr_names,
            attr_obj_map=attr_obj_map,
            node_ref=t,
            settings={
                objpkg.SETTINGS_OUTPUT_KEY: objpkg.SETTINGS_OUTPUT_INLINE
                if not kcl_info.isprivate_field(t.name)
                else objpkg.SETTINGS_OUTPUT_IGNORE
            },
            index_signature=index_sign_obj,
            func=objpkg.KCLCompiledFunctionObject(
                name=t.name,
                params=params,
            ),
        )
        self.schema_mapping[runtime_type] = schema_type
        return objpkg.KCLSchemaDefTypeObject(schema_type=schema_type)

    def build_rule_type(
        self,
        t: ast.RuleStmt,
        protocol_def: objpkg.KCLSchemaDefTypeObject = None,
        should_add_schema_ref: bool = False,
    ) -> objpkg.KCLSchemaDefTypeObject:
        """Build a schema type using the rule statement"""
        if t.name in RESERVED_TYPE_IDENTIFIERS:
            self.raise_err(
                [t],
                kcl_error.ErrType.IllegalInheritError_TYPE,
                "rule name '{}' cannot be the same as the built-in types ({})".format(
                    t.name, ", ".join(RESERVED_TYPE_IDENTIFIERS)
                ),
            )
        protocol = protocol_def.schema_type if protocol_def else None
        mixin_name_list = []
        for mixin in t.parent_rules or []:
            mixin_names = mixin.names
            if mixin.pkgpath:
                mixin_names[0] = f"@{mixin.pkgpath}"
            mixin_name_list.append(mixin.get_name())

        params: List[objpkg.Parameter] = []
        # Schema arguments
        if t.args:
            for i, arg in enumerate(t.args.args):
                type_annotation = t.args.GetArgType(i)
                type_node = self.parse_type_str_with_scope(
                    type_annotation, t.args.args[i]
                )
                default = t.args.GetArgDefault(i)
                params.append(
                    objpkg.Parameter(
                        name=arg.names[0],
                        value=objpkg.to_kcl_obj(default),
                        type_annotation=type_annotation,
                        type=type_node,
                    )
                )
                t.args.SetArgType(i, type_to_kcl_type_annotation_str(type_node))
        runtime_type = objpkg.KCLSchemaTypeObject.schema_runtime_type(
            t.name, self.pkgpath
        )

        return objpkg.KCLSchemaDefTypeObject(
            schema_type=objpkg.KCLSchemaTypeObject(
                name=t.name,
                is_rule=True,
                pkgpath=self.pkgpath,
                filename=self.filename,
                protocol=protocol,
                runtime_type=runtime_type,
                mixins_names=mixin_name_list,
                attr_list=[],
                attr_obj_map={},
                settings={
                    objpkg.SETTINGS_OUTPUT_KEY: objpkg.SETTINGS_OUTPUT_INLINE
                    if not kcl_info.isprivate_field(t.name)
                    else objpkg.SETTINGS_OUTPUT_IGNORE
                },
                func=objpkg.KCLCompiledFunctionObject(
                    name=t.name,
                    params=params,
                ),
            )
        )

    @staticmethod
    def is_builtin_or_plugin_module(path: str) -> bool:
        """Whether is a builtin system module or a plugin module"""
        if not path or not isinstance(path, str):
            return False
        return path in builtin.STANDARD_SYSTEM_MODULES or path.startswith(
            plugin.PLUGIN_MODULE_NAME
        )

    def init_global_types(self):
        """Init global types including top-level global variable types and schema types
        TODO: optimize the function with twice scan
        """
        # 1. Scan all schema type symbol
        for module in self.program.pkgs[self.pkgpath]:
            self.change_package_context(self.pkgpath, module.filename)
            for stmt in module.GetSchemaAndRuleList():
                if stmt.name in self.scope.elems:
                    self.raise_err(
                        [stmt],
                        kcl_error.ErrType.UniqueKeyError_TYPE,
                        kcl_error.UNIQUE_KEY_MSG.format(stmt.name),
                    )
                    continue
                schema_type_obj = objpkg.KCLSchemaTypeObject(
                    name=stmt.name,
                    runtime_type=objpkg.KCLSchemaTypeObject.schema_runtime_type(
                        stmt.name, self.pkgpath
                    ),
                    filename=self.filename,
                    pkgpath=self.pkgpath,
                )
                self.scope.elems[stmt.name] = ScopeObject(
                    name=stmt.name,
                    node=stmt,
                    type=objpkg.KCLSchemaDefTypeObject(
                        schema_type=schema_type_obj,
                    ),
                    pos=ast.Position(
                        filename=self.filename,
                        line=stmt.line,
                        column=stmt.column,
                    ),
                    end=ast.Position(
                        filename=self.filename,
                        line=stmt.end_line,
                        column=stmt.end_column,
                    ),
                )
        # 2. Scan all variable type symbol
        self.init_global_var_types()
        # 3. Build all schema types
        for i in range(MAX_SCOPE_SCAN_COUNT):
            for k, o in self.scope.elems.items():
                if isinstance(o.node, ast.SchemaStmt):
                    self.filename = o.type.schema_type.filename
                    schema_parent_type = self.build_schema_parent_type(
                        self.scope.elems[k].node
                    )
                    schema_protocol_type = self.build_schema_protocol_type(
                        self.scope.elems[k].node
                    )
                    self.scope.elems[k].type = self.build_schema_type(
                        self.scope.elems[k].node,
                        schema_parent_type,
                        schema_protocol_type,
                        i == MAX_SCOPE_SCAN_COUNT - 1,
                    )
                    self.scope.elems[k].type = cast(
                        objpkg.KCLSchemaDefTypeObject, self.scope.elems[k].type
                    )
                elif isinstance(o.node, ast.RuleStmt):
                    self.filename = o.type.schema_type.filename
                    schema_protocol_type = self.build_rule_protocol_type(
                        self.scope.elems[k].node
                    )
                    self.scope.elems[k].type = self.build_rule_type(
                        self.scope.elems[k].node,
                        schema_protocol_type,
                        i == MAX_SCOPE_SCAN_COUNT - 1,
                    )
                    self.scope.elems[k].type = cast(
                        objpkg.KCLSchemaDefTypeObject, self.scope.elems[k].type
                    )
        # 4. Build all variable types
        self.init_global_var_types(False)

    def do_import_stmt_check(self, t: ast.ImportStmt):
        """Do import check and store the module object into the map"""
        pkgpath = f"@{t.path}"
        for name in [pkgpath, t.pkg_name]:
            if name in self.scope.elems:
                self.scope.elems[name].type.imported_filenames.append(self.filename)
            else:
                module_object = objpkg.KCLModuleTypeObject(
                    pkgpath=t.path,
                    imported_filenames=[self.filename],
                    is_user_module=not self.is_builtin_or_plugin_module(t.path),
                    is_system_module=t.path in builtin.STANDARD_SYSTEM_MODULES,
                    is_plugin_module=t.path.startswith(plugin.PLUGIN_MODULE_NAME),
                )
                self.scope.elems[name] = ScopeObject(
                    name=t.path,
                    node=t,
                    type=module_object,
                    pos=ast.Position(
                        filename=self.filename,
                        line=t.line,
                        column=t.column,
                    ),
                    end=ast.Position(
                        filename=self.filename,
                        line=t.end_line,
                        column=t.end_column,
                    ),
                )
        if not self.scope.elems[pkgpath].type.is_user_module:
            return
        # Save current pkgpath and filename
        current_pkg_path = self.pkgpath
        current_filename = self.filename
        # Recursive import check
        if self.import_reference.add_node_judge_cycle(self.pkgpath, t.path):
            kcl_error.report_exception(
                err_type=kcl_error.ErrType.RecursiveLoad_TYPE,
                file_msgs=[
                    kcl_error.ErrFileMsg(
                        filename=self.filename,
                        line_no=t.line,
                    ),
                ],
                arg_msg=kcl_error.RECURSIVE_LOADING_MODULE_MSG.format(
                    current_pkg_path,
                    ", ".join([self.pkgpath, t.path]),
                ),
            )
        # Switch pkgpath context
        if t.path not in self.scope_map:
            self.check(t.path)
        # Restore the current context
        self.change_package_context(current_pkg_path, current_filename)

    def parse_type_str_with_scope(self, type_str: str, node: ast.AST) -> Type:
        # Type str to Type
        tpe = parse_type_str(type_str)

        # If a named type, find it from scope to get the specific type
        def walk_fn(t: Type):
            if isinstance(t, objpkg.KCLNamedTypeObject):
                if "." in t.name and t.name.rsplit(".", 1)[0] == f"@{self.pkgpath}":
                    t.name = t.name.replace(f"@{self.pkgpath}.", "", 1)
                tpe = self.expr(
                    ast.Identifier(
                        names=t.name.rsplit(".", 1)
                        if t.name.startswith("@")
                        else t.name.split("."),
                        line=node.line,
                    ).set_filename(node.filename)
                )
                if isinstance(tpe, objpkg.KCLSchemaDefTypeObject):
                    return tpe.schema_type
                elif isinstance(tpe, objpkg.KCLNumberMultiplierTypeObject):
                    return tpe
                elif hasattr(tpe, "is_type_alias") and tpe.is_type_alias:
                    return tpe
                else:
                    self.raise_err(
                        [node],
                        msg=f"'{t.name}' refers to a value, but is being used as a type here",
                    )
            return t

        return WalkType(tpe, walk_fn)

    def init_import_list(self):
        """Init import list and store the module scope object into the scope map"""
        for module in self.program.pkgs[self.pkgpath]:
            self.filename = module.filename
            import_stmt_list = module.GetImportList()
            for t in import_stmt_list:
                self.do_import_stmt_check(t)

    def init_global_var_types(self, unique_check: bool = True):
        """Init all global variable types"""

        def get_top_level_assign_list(module: ast.Module) -> List[ast.AssignStmt]:

            attr_list = []

            def loop_body(body: List[ast.Stmt]):
                """Get the l-values recursively and add them into schema attr list"""
                if not body:
                    return
                for stmt in body:
                    if isinstance(stmt, ast.AssignStmt):
                        attr_list.append(stmt)
                    elif isinstance(stmt, ast.IfStmt):
                        loop_body(stmt.body)
                        for body in stmt.elif_body:
                            loop_body(body)
                        loop_body(stmt.else_body)

            loop_body(module.body)
            return attr_list

        def init_scope_with_assign_stmt(assign_stmt: ast.AssignStmt):
            for target in assign_stmt.targets:
                name = target.names[0]
                if (
                    name in self.scope.elems
                    and not kcl_info.isprivate_field(name)
                    and unique_check
                ):
                    self.raise_err(
                        [target],
                        kcl_error.ErrType.ImmutableCompileError_TYPE,
                    )
                    continue
                if assign_stmt.type_annotation:
                    annotation_type = self.parse_type_str_with_scope(
                        assign_stmt.type_annotation, assign_stmt
                    )
                    assign_stmt.type_annotation = type_to_kcl_type_annotation_str(
                        annotation_type
                    )
                    if name in self.scope.elems:
                        origin_type = self.scope.elems[name].type
                        if not is_upper_bound(origin_type, annotation_type):
                            self.raise_err(
                                nodes=[self.scope.elems[name].node, target],
                                category=kcl_error.ErrType.TypeError_Compile_TYPE,
                                msg=f"can not change type of {name}",
                                file_msgs=[
                                    f"expect {origin_type.type_str()}",
                                    f"got {annotation_type.type_str()}",
                                ],
                                file_levels=[
                                    kcl_error.ErrLevel.ORDINARY,
                                    kcl_error.ErrLevel.SERIOUS,
                                ],
                            )
                            continue
                elif name in self.scope.elems:
                    annotation_type = self.scope.elems[name].type or ANY_TYPE
                else:
                    annotation_type = ANY_TYPE
                self.scope.elems[name] = ScopeObject(
                    name=name,
                    node=target,
                    type=annotation_type,
                    pos=ast.Position(
                        filename=self.filename,
                        line=target.line,
                        column=target.column,
                    ),
                    end=ast.Position(
                        filename=self.filename,
                        line=target.end_line,
                        column=target.end_column,
                    ),
                )

        for module in self.program.pkgs[self.pkgpath]:
            self.change_package_context(self.pkgpath, module.filename)
            for stmt in astutil.filter_stmt(module, ast.TypeAliasStmt):
                self.walk(stmt)
            for stmt in get_top_level_assign_list(module):
                init_scope_with_assign_stmt(stmt)

    def dict_assignable_to_schema(
        self,
        node: ast.AST,
        dict_type: Union[objpkg.KCLDictTypeObject, objpkg.KCLAnyTypeObject],
        schema_type: objpkg.KCLSchemaTypeObject,
        relaxed_key_type_mapping: Optional[Dict[str, Type]] = None,
    ) -> bool:
        """Judge a dict can be converted to schema in compile time"""
        # Do relaxed schema check key and value type check
        if relaxed_key_type_mapping and not False and not schema_type.index_signature:
            self.raise_err(
                nodes=[node],
                category=kcl_error.ErrType.CannotAddMembers_TYPE,
                msg=kcl_error.CANNOT_ADD_MEMBERS_MSG.format(
                    ",".join(relaxed_key_type_mapping.keys()), schema_type.name
                ),
            )
            return False
        if dict_type == ANY_TYPE:
            return True
        if schema_type.index_signature:
            schema_key_type = schema_type.index_signature.key_kcl_type
            schema_value_type = schema_type.index_signature.value_kcl_type
            for k in relaxed_key_type_mapping or {}:
                tpe = relaxed_key_type_mapping[k]
                if not assignable_to(tpe, schema_value_type):
                    self.raise_err(
                        [node],
                        msg=f"expected schema index signature value type {schema_value_type.type_str()}, "
                        f"got {tpe.type_str()} of the key '{k}'",
                    )
            if not schema_type.index_signature.any_other:
                return assignable_to(
                    dict_type.key_type, schema_key_type
                ) and assignable_to(dict_type.value_type, schema_value_type)
        return True

    def load_attr_type(
        self,
        node: ast.AST,
        obj: Type,
        attr: str,
    ) -> Type:
        if obj == ANY_TYPE:
            return ANY_TYPE
        if isinstance(obj, objpkg.KCLDictTypeObject):
            return obj.value_type
        if isinstance(obj, objpkg.KCLSchemaDefTypeObject):
            # Schema type member functions
            if attr in objpkg.KCLSchemaTypeObject.MEMBER_FUNCTIONS:
                return SCHEMA_TYPE_MEMBER_SCOPE.elems[attr].type
            self.raise_err(
                [node],
                kcl_error.ErrType.AttributeError_TYPE,
                f"schema '{obj.type_str()}' attribute '{attr}' not found",
            )
            return ANY_TYPE
        if isinstance(obj, objpkg.KCLSchemaTypeObject):
            # Schema attribute
            if obj.get_type_of_attr(attr) is None:
                if not obj.should_add_additional_key:
                    self.raise_err(
                        [node],
                        kcl_error.ErrType.AttributeError_TYPE,
                        f"schema '{obj.type_str()}' attribute '{attr}' not found",
                    )
                return ANY_TYPE
            return obj.get_type_of_attr(attr)
        if isinstance(obj, (objpkg.KCLStringTypeObject, objpkg.KCLStringLitTypeObject)):
            if attr not in objpkg.KCLStringObject.MEMBER_FUNCTIONS:
                self.raise_err(
                    [node],
                    kcl_error.ErrType.AttributeError_TYPE,
                    f"str object has no attribute '{attr}'",
                )
                return ANY_TYPE
            return STR_TYPE_MEMBER_SCOPE.elems[attr].type
        if isinstance(obj, objpkg.KCLUnionTypeObject):
            return ANY_TYPE
            # TODO: union type load attr based the type guard. e.g, a: str|int; if a is str: xxx; if a is int: xxx;
            # return sup([self.load_attr_type(t, attr, filename, line, column) for t in obj.types])
        self.raise_err(
            [node],
            kcl_error.ErrType.AttributeError_TYPE,
            f"{obj.type_str() if obj else None} has no attribute '{attr}'",
        )
        return ANY_TYPE

    def check_attr(
        self, node: ast.AST, obj: Type, attr: str, check_rules: List[typing.Callable]
    ):
        if obj and isinstance(obj, objpkg.KCLSchemaTypeObject):
            for check_rule in check_rules:
                check_rule(name=attr, node=node, schema_type=obj)

    def check_defined(
        self,
        name: Optional[str],
        node: ast.AST,
        schema_type: objpkg.KCLSchemaTypeObject,
    ):
        schema_type = self.schema_mapping.get(schema_type.runtime_type) or schema_type
        if (
            isinstance(schema_type, objpkg.KCLSchemaTypeObject)
            and not schema_type.get_obj_of_attr(name)
            and not schema_type.can_add_members()
            and not self._is_in_lambda_expr[-1]
        ):
            self.raise_err(
                nodes=[node],
                category=kcl_error.ErrType.CannotAddMembers_TYPE,
                msg=f"Cannot add member '{name}' to schema '{schema_type.name}'",
                file_msgs=[f"'{name}' is not defined in schema '{schema_type.name}'"],
            )

    def check_type(self, node: ast.AST, tpe: Type, expected_type: Type) -> bool:
        if type is None:
            return False
        if (
            tpe.type_kind() == objpkg.KCLTypeKind.ListKind
            and expected_type.type_kind() == objpkg.KCLTypeKind.ListKind
        ):
            return self.check_type(node, tpe.item_type, expected_type.item_type)
        elif (
            tpe.type_kind() == objpkg.KCLTypeKind.DictKind
            and expected_type.type_kind() == objpkg.KCLTypeKind.DictKind
        ):
            return self.check_type(
                node, tpe.key_type, expected_type.key_type
            ) and self.check_type(node, tpe.value_type, expected_type.value_type)
        elif tpe.type_kind() == objpkg.KCLTypeKind.UnionKind:
            return all([self.check_type(node, t, expected_type) for t in tpe.types])
        if (
            tpe.type_kind() == objpkg.KCLTypeKind.DictKind
            and expected_type.type_kind() == objpkg.KCLTypeKind.SchemaKind
        ):
            return self.dict_assignable_to_schema(node, tpe, expected_type)
        if expected_type.type_kind() == objpkg.KCLTypeKind.UnionKind:
            return any([self.check_type(node, tpe, t) for t in expected_type.types])
        else:
            return assignable_to(tpe, expected_type)

    def must_assignable_to(
        self,
        node: ast.AST,
        tpe: Type,
        expected_type: Type,
        err_category=kcl_error.ErrType.TypeError_Compile_TYPE,
        expected_node: ast.AST = None,
    ):
        expect_type_str = (
            expected_type.type_str() if expected_type is not None else None
        )
        tpe_str = tpe.type_str() if tpe is not None else None
        if tpe is None or not self.check_type(node, tpe, expected_type):
            self.raise_err(
                nodes=[expected_node, node] if expected_node else [node],
                category=err_category,
                msg=f"expect {expect_type_str}, got {tpe_str}",
                file_msgs=[f"expect {expect_type_str}", f"got {tpe_str}"]
                if expected_node
                else [f"got {tpe_str}"],
                file_levels=[kcl_error.ErrLevel.ORDINARY, kcl_error.ErrLevel.SERIOUS]
                if expected_node
                else [kcl_error.ErrLevel.SERIOUS],
            )

    def must_be_type(self, node: ast.AST, expected_type: Type):
        if node and isinstance(node, ast.AST):
            tpe = self.walk(node)
            self.must_assignable_to(node, tpe, expected_type)

    def enter_scope(self, node: ast.AST):
        if node and isinstance(node, ast.AST):
            scope = Scope(
                parent=self.scope,
                node=node,
                pos=ast.Position(
                    filename=self.filename,
                    line=node.line,
                    column=node.column,
                ),
                end=ast.Position(
                    filename=self.filename,
                    line=node.end_line,
                    column=node.end_column,
                ),
            )
        else:
            scope = Scope(self.scope, node)
        self.scope.children.append(scope)
        self.scope = scope

    def leave_scope(self):
        self.scope = self.scope.parent
        self._local_vars = []

    def stmts(self, stmts: List[ast.Stmt]):
        stmt_types = [self.stmt(stmt) for stmt in stmts or []]
        return stmt_types[-1] if stmt_types else ANY_TYPE

    def exprs(self, exprs: List[ast.Expr]):
        return [self.expr(expr) for expr in exprs or []]

    def expr(self, node: ast.Expr):
        return self.walk(node)

    def expr_or_any_type(self, node: ast.Expr):
        return self.walk(node) if node else ANY_TYPE

    def stmt(self, node: ast.Stmt):
        return self.walk(node)

    def lookup_type_from_scope(self, name: str, node: ast.AST) -> Optional[Type]:
        tpe = self.find_type_in_scope(name)
        if tpe:
            return tpe
        self.raise_err(
            [node], msg="name '{}' is not defined".format(name.replace("@", ""))
        )
        return ANY_TYPE

    def find_type_in_scope(self, name: str) -> Optional[Type]:
        scope = self.find_scope(name)
        return scope.type or ANY_TYPE if scope else None

    def find_scope(self, name: str) -> Optional[Scope]:
        scope = self.scope
        while scope and name not in scope.elems:
            scope = scope.parent
        if scope:
            return scope.elems[name] or None
        return None

    def set_type_to_scope(self, name: str, tpe: Type, node: ast.AST):
        if not name:
            return
        scope = self.scope
        while scope and name not in scope.elems:
            scope = scope.parent
        if scope:
            scope.elems[name].type = infer_to_variable_type(tpe)
            return
        self.raise_err([node], msg=f"name '{name}' is not defined")

    def do_arguments_type_check(
        self,
        args: List[ast.Expr],
        kwargs: List[ast.Keyword],
        params: List[objpkg.Parameter],
    ):
        """Do schema argument type check"""
        arg_types: List[Type] = self.exprs(args)
        kwarg_types: List[Tuple[str, Type]] = []
        check_table = set()
        for kw in kwargs or []:
            arg_name = kw.arg.names[0]
            if arg_name in check_table:
                self.raise_err(
                    [kw], msg=f"duplicated keyword argument {kw.arg.get_name()}"
                )
            check_table.add(arg_name)
            arg_value_type = self.expr(kw.value)
            kwarg_types.append((arg_name, arg_value_type))

        if params:
            for i, value in enumerate(arg_types):
                arg_name = params[i].name
                expected_type = params[i].type
                self.must_assignable_to(
                    args[i],
                    value,
                    expected_type,
                    kcl_error.ErrType.TypeError_Compile_TYPE,
                )
            for i, kwarg in enumerate(kwarg_types):
                arg_name, value = kwarg
                if arg_name not in [p.name for p in params]:
                    self.raise_err(
                        [kwargs[i]],
                        msg=f"arguments got an unexpected keyword argument '{arg_name}'",
                    )
                expected_types = [p.type for p in params if arg_name == p.name]
                expected_type = expected_types[0] if expected_types else ANY_TYPE
                self.must_assignable_to(
                    kwargs[i],
                    value,
                    expected_type,
                    kcl_error.ErrType.TypeError_Compile_TYPE,
                )

    def do_loop_type_check(
        self,
        t: ast.AST,
        target_node: ast.AST,
        key_name: str,
        val_name: str,
        iter_type: Type,
    ):
        """Do loop type check including quant and comp for expression"""
        if isinstance(iter_type, objpkg.KCLUnionTypeObject):
            types = iter_type.types
        else:
            types = [iter_type]
        key_type, value_type = ANY_TYPE, ANY_TYPE
        last_key_type, last_value_type = VOID_TYPE, VOID_TYPE
        for iter_type in types:
            if not isinstance(iter_type, ITER_TYPES):
                self.raise_err(
                    [t], msg=f"'{iter_type.type_str()}' object is not iterable"
                )
            if isinstance(iter_type, objpkg.KCLListTypeObject):
                # Two variables
                if val_name:
                    key_type, value_type = sup([INT_TYPE, last_key_type]), sup(
                        [iter_type.item_type, last_value_type]
                    )
                    self.set_type_to_scope(key_name, key_type, target_node)
                    self.set_type_to_scope(val_name, value_type, target_node)
                else:
                    key_type = sup([iter_type.item_type, last_key_type])
                    self.set_type_to_scope(key_name, key_type, target_node)
            elif isinstance(
                iter_type, (objpkg.KCLDictTypeObject, objpkg.KCLSchemaTypeObject)
            ):
                key_type, value_type = sup([iter_type.key_type, last_key_type]), sup(
                    [iter_type.value_type, last_value_type]
                )
                self.set_type_to_scope(key_name, key_type, target_node)
                self.set_type_to_scope(val_name, value_type, target_node)
            elif isinstance(
                iter_type, (objpkg.KCLStringTypeObject, objpkg.KCLStringLitTypeObject)
            ):
                if val_name:
                    key_type, value_type = sup([INT_TYPE, last_key_type]), sup(
                        [STR_TYPE, last_value_type]
                    )
                    self.set_type_to_scope(key_name, key_type, target_node)
                    self.set_type_to_scope(val_name, value_type, target_node)
                else:
                    key_type = sup([STR_TYPE, last_key_type])
                    self.set_type_to_scope(key_name, key_type, target_node)
            last_key_type, last_value_type = key_type, value_type


class TypeChecker(BaseTypeChecker):
    def walk_Module(self, t: ast.Module):
        return self.stmts(t.body)

    def walk_ExprStmt(self, t: ast.ExprStmt):
        expr_types = self.exprs(t.exprs)
        return expr_types[-1] if expr_types else ANY_TYPE

    def walk_AssertStmt(self, t: ast.AssertStmt):
        self.expr(t.test)
        # Check type in if_cond expression
        self.expr_or_any_type(t.if_cond)
        self.must_be_type(t.msg, STR_TYPE)
        return ANY_TYPE

    def walk_IfStmt(self, t: ast.IfStmt):
        self.expr(t.cond)
        self.stmts(t.body)
        for elif_cond, elif_body in zip(t.elif_cond, t.elif_body):
            self.expr(elif_cond)
            self.stmts(elif_body)
        self.stmts(t.else_body)
        return ANY_TYPE

    def walk_ImportStmt(self, t: ast.ImportStmt):
        """import <name> as <asname>"""
        # Add package name into the scope
        if t.path not in self.scope_map:
            self.do_import_stmt_check(t)
        return ANY_TYPE

    def walk_RuleStmt(self, t: ast.RuleStmt):
        self.in_schema_type = cast(
            objpkg.KCLSchemaDefTypeObject, self.lookup_type_from_scope(t.name, t)
        ).schema_type
        # Rule Decorators
        self.exprs(t.decorators)
        self.enter_scope(t)
        # Rule args
        for param in self.in_schema_type.func.params or []:
            self.scope.elems[param.name] = ScopeObject(
                name=param.name,
                node=t.args,
                type=param.type,
                pos=ast.Position(
                    filename=self.filename,
                    line=t.line,
                    column=t.column,
                ),
                end=ast.Position(
                    filename=self.filename,
                    line=t.end_line,
                    column=t.end_column,
                ),
            )
        # Rule check expressions
        self.exprs(t.checks)
        self.leave_scope()
        self.in_schema_type = None
        return ANY_TYPE

    def walk_SchemaStmt(self, t: ast.SchemaStmt):
        self.in_schema_type = cast(
            objpkg.KCLSchemaDefTypeObject, self.lookup_type_from_scope(t.name, t)
        ).schema_type
        # Schema Decorators
        self.exprs(t.decorators)
        self.enter_scope(t)
        # Schema args
        for param in self.in_schema_type.func.params or []:
            self.scope.elems[param.name] = ScopeObject(
                name=param.name,
                node=t.args,
                type=param.type,
                pos=ast.Position(
                    filename=self.filename,
                    line=t.line,
                    column=t.column,
                ),
                end=ast.Position(
                    filename=self.filename,
                    line=t.end_line,
                    column=t.end_column,
                ),
            )
        # Schema index signature
        if (
            self.in_schema_type.index_signature
            and self.in_schema_type.index_signature.key_name
        ):
            self.scope.elems[
                self.in_schema_type.index_signature.key_name
            ] = ScopeObject(
                name=self.in_schema_type.index_signature.key_name,
                node=t.index_signature,
                type=self.in_schema_type.index_signature.key_kcl_type,
                pos=ast.Position(
                    filename=self.filename,
                    line=t.index_signature.line,
                    column=t.index_signature.column,
                ),
                end=ast.Position(
                    filename=self.filename,
                    line=t.index_signature.end_line,
                    column=t.index_signature.end_column,
                ),
            )
        schema_attr_names = t.GetLeftIdentifierList()
        for name in schema_attr_names:
            if name not in self.scope.elems:
                self.scope.elems[name] = ScopeObject(
                    name=name,
                    node=None,
                    type=ANY_TYPE,
                )
        # Schema body
        self.stmts(t.body)
        # Schema check block
        self.exprs(t.checks)
        self.leave_scope()
        self.in_schema_type = None
        return ANY_TYPE

    def walk_SchemaAttr(self, t: ast.SchemaAttr):
        self._local_vars = []
        if "." in t.name:
            self.raise_err([t], msg="schema attribute can not be selected")
        expected_type = self.in_schema_type.get_type_of_attr(t.name) or ANY_TYPE
        # Schema attribute decorators
        self.exprs(t.decorators)
        self.scope.elems[t.name] = ScopeObject(
            name=t.name,
            node=t,
            type=expected_type,
            pos=ast.Position(
                filename=self.filename,
                line=t.line,
                column=t.column,
            ),
            end=ast.Position(
                filename=self.filename,
                line=t.end_line,
                column=t.end_column,
            ),
        )
        # Do not check type if no default value
        if t.value:
            if isinstance(expected_type, objpkg.KCLSchemaTypeObject):
                init_stack_depth = self.switch_config_expr_context(
                    self.new_config_expr_context_item(
                        name=expected_type.name, type_obj=expected_type, node=t
                    )
                )
                value_type = self.expr(t.value)
                self.clear_config_expr_context(stack_depth=init_stack_depth)
            else:
                value_type = self.expr(t.value)
            # Assign
            if not t.op or t.op == ast.AugOp.Assign:
                self.must_assignable_to(
                    t,
                    value_type,
                    expected_type,
                    kcl_error.ErrType.TypeError_Compile_TYPE,
                )
            else:
                self.must_assignable_to(
                    t,
                    binary(expected_type, value_type, t.op),
                    expected_type,
                    kcl_error.ErrType.TypeError_Compile_TYPE,
                )
        return ANY_TYPE

    def walk_Decorator(self, t: ast.Decorator):
        decorator_name = t.name.get_name()
        # Judge invalid decorator
        internal.decorator_factory.get(
            decorator_name,
            internal.DecoratorTargetType.SCHEMA_TYPE,
            filename=self.filename,
            lineno=t.line,
            columnno=t.column,
        )
        # Decorator args type check according to decorator lookup table
        decorator_func_type = DECORATOR_SCOPE.elems.get(decorator_name, ANY_TYPE).type
        if isinstance(decorator_func_type, objpkg.KCLFunctionTypeObject) and t.args:
            self.do_arguments_type_check(
                t.args.args, t.args.keywords, decorator_func_type.params
            )
        return ANY_TYPE

    def walk_IfExpr(self, t: ast.IfExpr):
        """<body> if <cond> else <orelse> -> sup([body, orelse])"""
        self.expr(t.cond)
        types = self.exprs([t.body, t.orelse])
        return sup(types)

    def walk_UnaryExpr(self, t: ast.UnaryExpr):
        return unary(self.expr(t.operand), t.op, self.filename, t.line)

    def walk_BinaryExpr(self, t: ast.BinaryExpr):
        left_type, right_type = self.expr(t.left), self.expr(t.right)
        if t.op == ast.BinOp.As:
            if not isinstance(t.right, ast.Identifier):
                self.raise_err(
                    [t.right], msg="Keyword 'as' right operand must be a type"
                )
                return self.expr(t.left)
            # Replace with type alias
            right_type = self.parse_type_str_with_scope(t.right.get_name(), t.right)
            type_annotation_str = type_to_kcl_type_annotation_str(right_type)
            if (
                "." in type_annotation_str
                and type_annotation_str.rsplit(".", 1)[0] == f"@{self.pkgpath}"
            ):
                type_annotation_str = type_annotation_str.replace(
                    f"@{self.pkgpath}.", "", 1
                )
            t.right.names = type_annotation_str.rsplit(".", 1)
        return binary(left_type, right_type, t.op, self.filename, t.line)

    def walk_Compare(self, t: ast.Compare):
        for t1, t2, op in zip([t.left] + t.comparators, t.comparators, t.ops):
            compare(self.expr(t1), self.expr(t2), op, self.filename, t.line)
        return BOOL_TYPE

    def walk_SelectorExpr(self, t: ast.SelectorExpr):
        value_type = self.expr(t.value)
        for name in t.attr.names:
            value_type = self.load_attr_type(t, value_type, name)
        return value_type

    def walk_CallExpr(self, t: ast.CallExpr):
        func_type = self.expr(t.func)
        if (
            func_type != ANY_TYPE
            and func_type.type_kind() != objpkg.KCLTypeKind.FuncKind
            and not isinstance(func_type, objpkg.KCLSchemaDefTypeObject)
        ):
            self.raise_err([t], msg=f"'{func_type.type_str()}' object is not callable")
            return ANY_TYPE
        if func_type == ANY_TYPE:
            self.do_arguments_type_check(
                t.args,
                t.keywords,
                [],
            )
            return ANY_TYPE
        self.do_arguments_type_check(
            t.args,
            t.keywords,
            func_type.schema_type.func.params
            if isinstance(func_type, objpkg.KCLSchemaDefTypeObject)
            else func_type.params,
        )
        return (
            func_type.schema_type
            if isinstance(func_type, objpkg.KCLSchemaDefTypeObject)
            else func_type.return_type
        )

    def walk_ParenExpr(self, t: ast.ParenExpr):
        return self.expr(t.expr)

    def walk_QuantExpr(self, t: ast.QuantExpr):
        """
        self.target: Optional[Expr] = None
        self.variables: List[Identifier] = []
        self.op: Optional[int] = None
        self.test: Optional[Expr] = None
        self.if_cond: Optional[Expr] = None
        """
        iter_type = self.expr(t.target)
        if iter_type == ANY_TYPE:
            return ANY_TYPE
        self.enter_scope(t)
        key_name = None
        val_name = None
        target_node = None
        for i, target in enumerate(t.variables):
            target_node = target
            name = target.names[0]
            key_name = name if i == 0 else key_name
            val_name = name if i == 1 else val_name
            self._local_vars.append(name)
            self.scope.elems[name] = ScopeObject(
                name=name,
                node=target,
                type=ANY_TYPE,
            )
        self.do_loop_type_check(t, target_node, key_name, val_name, iter_type)
        self.expr_or_any_type(t.if_cond)
        item_type = self.expr(t.test)
        return_type = ANY_TYPE
        if t.op in [ast.QuantOperation.ALL, ast.QuantOperation.ANY]:
            return_type = BOOL_TYPE
        elif t.op == ast.QuantOperation.MAP:
            return_type = objpkg.KCLListTypeObject(item_type=item_type)
        elif t.op == ast.QuantOperation.FILTER:
            return_type = iter_type
        else:
            self.raise_err([t], msg=f"Invalid quantifier expression op {t.op}")
        self.leave_scope()
        return return_type

    def walk_ListExpr(self, t: ast.ListExpr):
        item_type = sup(self.exprs(t.elts))
        return objpkg.KCLListTypeObject(item_type=item_type)

    def walk_StarredExpr(self, t: ast.StarredExpr):
        """Single star unpack expression *t.value"""
        value_type = self.expr(t.value)
        if value_type == ANY_TYPE:
            return ANY_TYPE
        if isinstance(value_type, objpkg.KCLNoneTypeObject):
            return NONE_TYPE
        if isinstance(value_type, objpkg.KCLListTypeObject):
            return value_type.item_type
        if isinstance(
            value_type, (objpkg.KCLDictTypeObject, objpkg.KCLSchemaTypeObject)
        ):
            return value_type.key_type
        if is_kind_type_or_kind_union_type(
            value_type,
            [objpkg.KCLTypeKind.DictKind, objpkg.KCLTypeKind.SchemaKind],
        ):
            return sup([tpe.key_type for tpe in value_type.types])
        self.raise_err(
            [t.value],
            msg=f"only list, dict, schema object can be used * unpacked, got {value_type.type_str()}",
        )
        return ANY_TYPE

    def walk_ListComp(self, t: ast.ListComp):
        self.enter_scope(t)
        self.exprs(t.generators)
        if isinstance(t.elt, ast.StarredExpr):
            self.raise_err(
                [t.elt],
                msg="list unpacking cannot be used in list comprehension",
            )
        item_type = self.expr(t.elt)
        self.leave_scope()
        return objpkg.KCLListTypeObject(item_type=item_type)

    def walk_DictComp(self, t: ast.DictComp):
        self.enter_scope(t)
        self.exprs(t.generators)
        if not t.key:
            self.raise_err(
                [t.value],
                msg="dict unpacking cannot be used in dict comprehension",
            )
        key_type, value_type = self.expr(t.key), self.expr(t.value)
        self.leave_scope()
        return objpkg.KCLDictTypeObject(key_type=key_type, value_type=value_type)

    def walk_CompClause(self, t: ast.CompClause):
        iter_type = self.expr(t.iter)
        key_name = None
        val_name = None
        target_node = None
        for i, target in enumerate(t.targets):
            target_node = target
            name = target.names[0]
            key_name = name if i == 0 else key_name
            val_name = name if i == 1 else val_name
            self._local_vars.append(name)
            self.scope.elems[name] = ScopeObject(
                name=name,
                node=target,
                type=ANY_TYPE,
                pos=ast.Position(
                    filename=self.filename,
                    line=target.line,
                    column=target.column,
                ),
                end=ast.Position(
                    filename=self.filename,
                    line=target.end_line,
                    column=target.end_column,
                ),
            )
        if iter_type == ANY_TYPE:
            return ANY_TYPE
        self.do_loop_type_check(t, target_node, key_name, val_name, iter_type)
        self.exprs(t.ifs)
        return ANY_TYPE

    def walk_Subscript(self, t: ast.Subscript):
        value_type = self.expr(t.value)
        if value_type == ANY_TYPE:
            return ANY_TYPE
        if isinstance(
            value_type, (objpkg.KCLSchemaTypeObject, objpkg.KCLDictTypeObject)
        ):
            if not t.index:
                self.raise_err([t], msg="unhashable type: 'slice'")
                return ANY_TYPE
            key_type = self.expr(t.index)
            if key_type == ANY_TYPE or key_type == NONE_TYPE:
                return value_type.value_type
            if not is_kind_type_or_kind_union_type(key_type, KEY_KINDS):
                self.raise_err(
                    [t.index],
                    msg=f"invalid dict/schema key type: '{key_type.type_str()}'",
                )
                return ANY_TYPE
            return (
                self.load_attr_type(t, value_type, t.index.value)
                if isinstance(t.index, ast.StringLit)
                else value_type.value_type
            )
        elif isinstance(
            value_type,
            (
                objpkg.KCLListTypeObject,
                objpkg.KCLStringTypeObject,
                objpkg.KCLStringLitTypeObject,
            ),
        ):
            if t.index:
                self.must_be_type(t.index, INT_TYPE)
                return (
                    value_type.item_type
                    if isinstance(value_type, objpkg.KCLListTypeObject)
                    else STR_TYPE
                )
            else:
                self.must_be_type(t.lower, INT_TYPE)
                self.must_be_type(t.upper, INT_TYPE)
                self.must_be_type(t.step, INT_TYPE)
                return (
                    value_type
                    if isinstance(value_type, objpkg.KCLListTypeObject)
                    else STR_TYPE
                )
        self.raise_err(
            [t.value], msg=f"'{value_type.type_str()}' object is not subscriptable"
        )
        return ANY_TYPE

    def walk_SchemaExpr(self, t: ast.SchemaExpr):
        # Auto append schema import statements.
        if self.config.config_attr_auto_fix:
            try:
                schema_def_type = self.walk(t.name)
            except Exception:
                # Match the schema package path.
                pkgpaths = [
                    schema.pkgpath
                    for schema in self.schema_mapping.values()
                    if t.name.get_name() == schema.name
                ]
                if pkgpaths:
                    # Select the first matched path.
                    pkgpath = pkgpaths[0]
                    import_list = self.program.pkgs[self.pkgpath][0].GetImportList()
                    imported_pkgpath = [stmt.path for stmt in import_list]
                    # Exists import
                    try:
                        index = imported_pkgpath.index(pkgpath)
                        t.name.names = [import_list[index].pkg_name] + t.name.names
                    # Not exists import, append the import stmt.
                    except ValueError:
                        if pkgpath and pkgpath != ast.Program.MAIN_PKGPATH:
                            name = pkgpath.rsplit(".")[-1]
                            import_stmt = ast.ImportStmt(1, 1)
                            import_stmt.path = pkgpath
                            import_stmt.name = name
                            self.program.pkgs[self.pkgpath][0].body.insert(
                                0, import_stmt
                            )
                            t.name.names = [name] + t.name.names
                return ANY_TYPE
        else:
            schema_def_type = self.walk(t.name)
        if isinstance(schema_def_type, objpkg.KCLSchemaDefTypeObject):
            schema_type_annotation_str = type_to_kcl_type_annotation_str(
                schema_def_type
            )
            if (
                "." in schema_type_annotation_str
                and schema_type_annotation_str.rsplit(".", 1)[0] == f"@{self.pkgpath}"
            ):
                schema_type_annotation_str = schema_type_annotation_str.replace(
                    f"@{self.pkgpath}.", "", 1
                )
            t.name.names = schema_type_annotation_str.rsplit(".", 1)
            schema_type = cast(objpkg.KCLSchemaTypeObject, schema_def_type.schema_type)
            init_stack_depth = self.switch_config_expr_context(
                self.new_config_expr_context_item(
                    name=schema_type.name, type_obj=schema_type, node=t
                )
            )
            self.expr(t.config)
            self.clear_config_expr_context(stack_depth=init_stack_depth)
            # Do schema argument type check
            self.do_arguments_type_check(t.args, t.kwargs, schema_type.func.params)
            return schema_type
        elif isinstance(schema_def_type, objpkg.KCLSchemaTypeObject):
            schema_type = schema_def_type
            init_stack_depth = self.switch_config_expr_context(
                self.new_config_expr_context_item(
                    name=schema_type.name, type_obj=schema_type, node=t
                )
            )
            self.expr(t.config)
            self.clear_config_expr_context(stack_depth=init_stack_depth)
            if t.args or t.kwargs:
                self.raise_err(
                    [t.name],
                    msg="Arguments cannot be used in the schema modification expression",
                )
            return schema_type
        elif isinstance(schema_def_type, objpkg.KCLDictTypeObject):
            dict_type = schema_def_type
            init_stack_depth = self.switch_config_expr_context(
                self.new_config_expr_context_item(type_obj=dict_type, node=t)
            )
            config_type = self.expr(t.config)
            self.clear_config_expr_context(stack_depth=init_stack_depth)
            return binary(
                dict_type, config_type, ast.BinOp.BitOr, self.filename, t.line
            )
        self.raise_err(
            [t.name],
            msg=f"Invalid schema type '{schema_def_type.type_str() if schema_def_type else None}'",
        )
        return ANY_TYPE

    def walk_ConfigExpr(self, t: ast.ConfigExpr):
        value_types = []
        key_types = []
        fix_values = []
        for i, key, value, op in zip(
            range(len(t.keys)), t.keys, t.values, t.operations
        ):
            stack_depth = 0
            fix_value = self.check_config_entry(key, value, [self.check_defined])
            fix_values.append((i, fix_value))
            stack_depth += self.switch_config_expr_context_by_key(key)
            value_type = ANY_TYPE
            has_insert_index = False
            # Double_star expr and dict_if_entry expr
            if not key:
                value_type = self.expr(value)
                if value_type == ANY_TYPE or value_type == NONE_TYPE:
                    value_types.append(value_type)
                elif isinstance(
                    value_type, (objpkg.KCLDictTypeObject, objpkg.KCLSchemaTypeObject)
                ):
                    key_types.append(value_type.key_type)
                    value_types.append(value_type.value_type)
                elif is_kind_type_or_kind_union_type(
                    value_type,
                    [objpkg.KCLTypeKind.DictKind, objpkg.KCLTypeKind.SchemaKind],
                ):
                    key_types.append(sup([tpe.key_type for tpe in value_type.types]))
                    value_types.append(
                        sup([tpe.value_type for tpe in value_type.types])
                    )
                else:
                    self.raise_err(
                        nodes=[value],
                        msg=f"only dict and schema can be used ** unpack, got '{value_type.type_str()}'",
                    )
            elif isinstance(key, ast.Identifier):
                # Nest identifier key -> str key
                if len(key.names) == 1:
                    name = key.get_name(False)
                    key_type = self.expr(key) if name in self._local_vars else STR_TYPE
                    if key_type != ANY_TYPE and key_type.type_kind() not in KEY_KINDS:
                        self.raise_err(
                            nodes=[key],
                            category=kcl_error.ErrType.IllegalAttributeError_TYPE,
                            msg=f"type '{key_type.type_str()}'",
                        )
                else:
                    key_type = STR_TYPE
                key_types.append(key_type)
                value_type = self.expr(value)
                nest_value_type = value_type
                for _ in range(len(key.names) - 1):
                    nest_value_type = objpkg.KCLDictTypeObject(
                        key_type=STR_TYPE, value_type=nest_value_type
                    )
                value_types.append(nest_value_type)
            elif isinstance(key, ast.Subscript):
                if isinstance(key.value, ast.Identifier) and isinstance(
                    key.index, ast.NumberLit
                ):
                    has_insert_index = True
                    value_type = self.expr(value)
                    key_types.append(STR_TYPE)
                    value_types.append(objpkg.KCLListTypeObject(value_type))
            else:
                key_type, value_type = self.expr(key), self.expr(value)
                if key_type != ANY_TYPE and key_type.type_kind() not in KEY_KINDS:
                    self.raise_err(
                        [key],
                        kcl_error.ErrType.IllegalAttributeError_TYPE,
                        f"type '{key_type.type_str()}'",
                    )
                key_types.append(key_type)
                value_types.append(value_type)
            if (
                op == ast.ConfigEntryOperation.INSERT
                and not has_insert_index
                and value_type != ANY_TYPE
                and not isinstance(value_type, objpkg.KCLListTypeObject)
            ):
                self.raise_err(
                    [value],
                    kcl_error.ErrType.IllegalAttributeError_TYPE,
                    f"only list type can in inserted, got '{value_type.type_str()}'",
                )
            self.clear_config_expr_context(stack_depth=stack_depth)
        key_type = sup(key_types)
        value_type = sup(value_types)
        for i, fix_value in fix_values:
            if fix_value:
                t.items[i].value = fix_value
        # self.clear_config_expr_context(stack_depth=init_stack_depth)
        return objpkg.KCLDictTypeObject(key_type=key_type, value_type=value_type)

    def walk_CheckExpr(self, t: ast.CheckExpr):
        self.must_be_type(t.msg, STR_TYPE)
        # Check type in if_cond expression
        self.expr_or_any_type(t.if_cond)
        return self.expr(t.test)

    def walk_LambdaExpr(self, t: ast.LambdaExpr):
        """ast.AST: LambdaExpr

        Parameters
        ----------
        - args: Optional[Arguments]
        - return_type_str: Optional[str]
        - return_type_node: Optional[Type]
        - body: List[Stmt]
        """
        params = []
        return_type = ANY_TYPE
        if t.args:
            for i, arg in enumerate(t.args.args):
                name = arg.get_name()
                type_annotation = t.args.GetArgType(i)
                type_node = self.parse_type_str_with_scope(
                    type_annotation, t.args.args[i]
                )
                default = t.args.GetArgDefault(i)
                params.append(
                    objpkg.Parameter(
                        name=name,
                        value=objpkg.to_kcl_obj(default),
                        type_annotation=type_annotation,
                        type=type_node,
                    )
                )
                t.args.SetArgType(i, type_to_kcl_type_annotation_str(type_node))
        if t.return_type_str:
            return_type = self.parse_type_str_with_scope(t.return_type_str, t)
            t.return_type_str = type_to_kcl_type_annotation_str(return_type)
        self.enter_scope(t)
        self._is_in_lambda_expr.append(True)
        # Lambda args
        for param in params or []:
            self.scope.elems[param.name] = ScopeObject(
                name=param.name,
                node=t.args,
                type=param.type,
                pos=ast.Position(
                    filename=self.filename,
                    line=t.line,
                    column=t.column,
                ),
                end=ast.Position(
                    filename=self.filename,
                    line=t.end_line,
                    column=t.end_column,
                ),
            )
        real_return_type = self.stmts(t.body)
        self.leave_scope()
        self._is_in_lambda_expr.pop()
        self.must_assignable_to(
            t.body[-1] if t.body else t, real_return_type, return_type
        )
        if (
            real_return_type != ANY_TYPE
            and return_type == ANY_TYPE
            and not t.return_type_str
        ):
            return_type = real_return_type
        return objpkg.KCLFunctionTypeObject(
            name="",
            params=params,
            self_type=ANY_TYPE,
            return_type=return_type,
            doc="",
        )

    def walk_Identifier(self, t: ast.Identifier):
        assert len(t.names) >= 1
        if t.pkgpath:
            t.names[0] = f"@{t.pkgpath}"
            if t.ctx == ast.ExprContext.STORE or t.ctx == ast.ExprContext.AUGSTORE:
                self.raise_err(
                    [t], msg="only schema and dict object can be updated attribute"
                )
        name = t.names[0]
        if os.getenv("KCL_FEATURE_GATEWAY_STRONG_MUTABLE"):
            if t.ctx == ast.ExprContext.STORE or t.ctx == ast.ExprContext.AUGSTORE:
                if self.in_schema_type:
                    if not kcl_info.isprivate_field(name) and (
                        name in self.scope.elems
                        and self.scope.elems[name].pos
                        and (
                            t.line != self.scope.elems[name].pos.line
                            or t.column != self.scope.elems[name].pos.column
                        )
                    ):
                        self.raise_err(
                            [t],
                            kcl_error.ErrType.ImmutableCompileError_TYPE,
                        )
        if len(t.names) == 1:
            if self.in_schema_type:
                # Load from schema if in schema
                tpe = self.in_schema_type.get_type_of_attr(name)
                if t.ctx == ast.ExprContext.LOAD or t.ctx == ast.ExprContext.AUGLOAD:
                    scope_type = self.find_type_in_scope(name)
                    if name in self._local_vars:
                        return scope_type or ANY_TYPE
                    if tpe and tpe != ANY_TYPE:
                        return tpe
                    return scope_type or ANY_TYPE
                    # TODO: Enhanced Mixin with protocol
                    # return tpe or self.lookup_type_from_scope(name, t)
                elif (
                    t.ctx == ast.ExprContext.STORE or t.ctx == ast.ExprContext.AUGSTORE
                ):
                    if name not in self.scope.elems or not tpe:
                        self.scope.elems[name] = ScopeObject(
                            name=name,
                            node=t,
                            type=ANY_TYPE,
                            pos=ast.Position(
                                filename=self.filename,
                                line=t.line,
                                column=t.column,
                            ),
                            end=ast.Position(
                                filename=self.filename,
                                line=t.end_line,
                                column=t.end_column,
                            ),
                        )
                        if not tpe:
                            self.in_schema_type.set_type_of_attr(name, ANY_TYPE)
                        return ANY_TYPE
                    self.check_attr(
                        t,
                        self.in_schema_type,
                        name,
                        [self.check_defined],
                    )
                    return tpe if tpe else self.lookup_type_from_scope(name, t)
            else:
                if t.ctx == ast.ExprContext.LOAD or t.ctx == ast.ExprContext.AUGLOAD:
                    return self.lookup_type_from_scope(name, t)
                elif (
                    t.ctx == ast.ExprContext.STORE or t.ctx == ast.ExprContext.AUGSTORE
                ):
                    if name not in self.scope.elems and not self.in_schema_type:
                        self.scope.elems[name] = ScopeObject(
                            name=name,
                            node=t,
                            type=ANY_TYPE,
                            pos=ast.Position(
                                filename=self.filename,
                                line=t.line,
                                column=t.column,
                            ),
                            end=ast.Position(
                                filename=self.filename,
                                line=t.end_line,
                                column=t.end_column,
                            ),
                        )
                        return ANY_TYPE
                    return self.lookup_type_from_scope(name, t)
            return ANY_TYPE
        else:
            names = t.names
            if t.ctx != ast.ExprContext.AUGSTORE:
                tpe = self.expr(
                    ast.Identifier(
                        names=[names[0]], line=t.line, column=t.column
                    ).set_filename(t.filename)
                )
            else:
                tpe = self.lookup_type_from_scope(t.names[0], t)

            for name in names[1:]:
                if isinstance(tpe, objpkg.KCLModuleTypeObject):
                    if tpe.is_user_module:
                        if tpe.pkgpath not in self.scope_map:
                            self.raise_err(
                                [t], msg=f"name '{tpe.pkgpath}' is not defined"
                            )
                            return ANY_TYPE
                        elif name not in self.scope_map[tpe.pkgpath].elems:
                            self.raise_err(
                                [t],
                                kcl_error.ErrType.AttributeError_TYPE,
                                f"module '{tpe.pkgpath}' has no attribute '{name}'",
                            )
                            tpe = ANY_TYPE
                        elif (
                            self.filename not in tpe.imported_filenames
                            and self.pkgpath != ast.Program.MAIN_PKGPATH
                        ):
                            self.raise_err(
                                [t], msg=f"name '{tpe.pkgpath}' is not defined"
                            )
                            tpe = ANY_TYPE
                        else:
                            tpe = self.scope_map[tpe.pkgpath].elems[name].type
                        if isinstance(tpe, objpkg.KCLModuleTypeObject):
                            self.raise_err(
                                [t],
                                kcl_error.ErrType.CompileError_TYPE,
                                f"can not import the attribute '{name}' from the module '{t.names[0]}'",
                                file_msgs=[f"'{name}' is a module attribute"],
                            )
                    elif tpe.is_plugin_module:
                        tpe = (
                            PLUGIN_SCOPE_MAPPING[tpe.pkgpath].elems[name].type
                            if (
                                tpe.pkgpath in PLUGIN_SCOPE_MAPPING
                                and name in PLUGIN_SCOPE_MAPPING[tpe.pkgpath].elems
                            )
                            else ANY_TYPE
                        )
                    elif tpe.is_system_module:
                        members = builtin.get_system_module_members(tpe.pkgpath)
                        if name not in members:
                            self.raise_err(
                                [t],
                                kcl_error.ErrType.AttributeError_TYPE,
                                f"module '{tpe.pkgpath}' has no attribute '{name}'",
                            )
                        tpe = (
                            MODULE_SCOPE_MAPPING[tpe.pkgpath].elems[name].type
                            if (
                                tpe.pkgpath in MODULE_SCOPE_MAPPING
                                and name in MODULE_SCOPE_MAPPING[tpe.pkgpath].elems
                            )
                            else ANY_TYPE
                        )
                    else:
                        tpe = ANY_TYPE
                elif (
                    t.ctx == ast.ExprContext.STORE or t.ctx == ast.ExprContext.AUGSTORE
                ):
                    self.check_attr(
                        t,
                        tpe,
                        name,
                        [self.check_defined],
                    )
                    tpe = self.load_attr_type(t, tpe, name)
                else:
                    tpe = self.load_attr_type(t, tpe, name)

            return tpe

    def walk_NumberLit(self, t: ast.AST):
        if t.binary_suffix:
            value = units.cal_num(t.value, t.binary_suffix)
            return objpkg.KCLNumberMultiplierTypeObject(
                value=value,
                raw_value=t.value,
                binary_suffix=t.binary_suffix,
            )
        return (
            objpkg.KCLIntLitTypeObject(t.value)
            if isinstance(t.value, int)
            else objpkg.KCLFloatLitTypeObject(t.value)
        )

    def walk_StringLit(self, t: ast.StringLit):
        return objpkg.KCLStringLitTypeObject(t.value)

    def walk_NameConstantLit(self, t: ast.NameConstantLit):
        if t.value is None or isinstance(t.value, internal.UndefinedType):
            return NONE_TYPE
        return TRUE_LIT_TYPE if t.value is True else FALSE_LIT_TYPE

    def walk_FormattedValue(self, t: ast.FormattedValue):
        if (
            t.format_spec
            and isinstance(t.format_spec, str)
            and t.format_spec.lower() not in VALID_FORMAT_SPEC_SET
        ):
            self.raise_err([t], msg=f"{t.format_spec} is a invalid format spec")
        return self.expr(t.value)

    def walk_JoinedString(self, t: ast.JoinedString):
        self.exprs(t.values)
        return STR_TYPE

    def walk_TypeAliasStmt(self, t: ast.TypeAliasStmt):
        """ast.AST: TypeAliasStmt

        Parameters
        ----------
        - type_name: Identifier
        - type_value: Type
        """
        tpe = self.parse_type_str_with_scope(t.type_value.plain_type_str, t.type_value)
        if isinstance(tpe, objpkg.KCLSchemaTypeObject):
            tpe = objpkg.KCLSchemaDefTypeObject(schema_type=tpe)
        tpe.is_type_alias = True
        name = t.type_name.get_name()
        if name in RESERVED_TYPE_IDENTIFIERS:
            self.raise_err(
                [t],
                kcl_error.ErrType.IllegalInheritError_TYPE,
                "type alias '{}' cannot be the same as the built-in types ({})".format(
                    name, ", ".join(RESERVED_TYPE_IDENTIFIERS)
                ),
            )
        self.scope.elems[name] = ScopeObject(
            name=name,
            node=t,
            type=tpe,
            pos=ast.Position(
                filename=self.filename,
                line=t.line,
                column=t.column,
            ),
            end=ast.Position(
                filename=self.filename,
                line=t.end_line,
                column=t.end_column,
            ),
        )
        return tpe

    def walk_UnificationStmt(self, t: ast.UnificationStmt):
        if len(t.target.names) > 1:
            self.raise_err([t.target], msg="unification identifier can not be selected")
        name = t.target.names[0]
        tpe = self.expr(t.target)
        value_tpe = self.expr(t.value)
        self.must_assignable_to(t.target, value_tpe, tpe)
        if value_tpe != ANY_TYPE and tpe == ANY_TYPE:
            self.set_type_to_scope(name, value_tpe, t.target)
        return value_tpe

    def walk_AssignStmt(self, t: ast.AssignStmt):
        """id: T = E"""
        self._local_vars = []
        for target in t.targets:
            name = target.names[0]
            if len(target.names) == 1:
                tpe = self.expr(target)
                if isinstance(tpe, objpkg.KCLSchemaTypeObject):
                    init_stack_depth = self.switch_config_expr_context(
                        self.new_config_expr_context_item(
                            name=tpe.name, type_obj=tpe, node=t
                        )
                    )
                    value_tpe = self.expr(t.value)
                    self.clear_config_expr_context(stack_depth=init_stack_depth)
                else:
                    value_tpe = self.expr(t.value)
                    self.must_assignable_to(target, value_tpe, tpe)
                if value_tpe != ANY_TYPE and tpe == ANY_TYPE and not t.type_annotation:
                    self.set_type_to_scope(name, value_tpe, target)
                    if self.in_schema_type:
                        # Set attr type if in schema
                        tpe = self.in_schema_type.set_type_of_attr(
                            name, infer_to_variable_type(value_tpe)
                        )
            else:
                self.lookup_type_from_scope(name, target)
                tpe = self.expr(target)
                value_tpe = self.expr(t.value)
                self.must_assignable_to(target, value_tpe, tpe)
        return value_tpe

    def walk_AugAssignStmt(self, t: ast.AugAssignStmt):
        t.target.ctx = ast.ExprContext.LOAD
        new_target_type = binary(
            self.expr(t.target), self.expr(t.value), t.op, self.filename, t.line
        )
        t.target.ctx = ast.ExprContext.STORE
        expected_type = self.expr(t.target)
        self.must_assignable_to(t.target, new_target_type, expected_type)
        return new_target_type

    def walk_ListIfItemExpr(self, t: ast.ListIfItemExpr):
        self.expr_or_any_type(t.if_cond)
        or_else_type = self.expr_or_any_type(t.orelse)
        # `orelse` node maybe a list unpack node, use its item type instead.
        if isinstance(or_else_type, objpkg.KCLListTypeObject):
            or_else_type = or_else_type.item_type
        exprs_type = sup(self.exprs(t.exprs))
        return sup([or_else_type, exprs_type])

    def walk_ConfigIfEntryExpr(self, t: ast.ConfigIfEntryExpr):
        self.expr_or_any_type(t.if_cond)
        key_types, value_types = [], []
        for key, value in zip(t.keys, t.values):
            stack_depth = 0
            self.check_config_entry(key, value, [self.check_defined])
            stack_depth += self.switch_config_expr_context_by_key(key)
            if not key:
                key_type = ANY_TYPE
                value_type = self.expr(value)
                if value_type == ANY_TYPE or value_type == NONE_TYPE:
                    value_types.append(value_type)
                elif isinstance(
                    value_type, (objpkg.KCLDictTypeObject, objpkg.KCLSchemaTypeObject)
                ):
                    key_type = value_type.key_type
                    value_type = value_type.value_type
                elif is_kind_type_or_kind_union_type(
                    value_type,
                    [objpkg.KCLTypeKind.DictKind, objpkg.KCLTypeKind.SchemaKind],
                ):
                    key_type = sup([tpe.key_type for tpe in value_type.types])
                    value_type = sup([tpe.value_type for tpe in value_type.types])
                else:
                    self.raise_err(
                        nodes=[value],
                        msg=f"only dict and schema can be used ** unpack, got '{value_type.type_str()}'",
                    )
            elif isinstance(key, ast.Identifier):
                key_type = STR_TYPE
                value_type = self.expr(value)
                for _ in range(len(key.names) - 1):
                    value_type = objpkg.KCLDictTypeObject(
                        key_type=STR_TYPE, value_type=value_type
                    )
            else:
                key_type = self.expr(key)
                value_type = self.expr(value)
            key_types.append(key_type)
            value_types.append(value_type)
            self.clear_config_expr_context(stack_depth=stack_depth)
        dict_type = objpkg.KCLDictTypeObject(
            key_type=sup(key_types),
            value_type=sup(value_types),
        )
        or_else_type = self.expr_or_any_type(t.orelse)
        return sup([dict_type, or_else_type])


def ResolveProgramImport(prog: ast.Program):
    """Check import error"""
    if not prog or not isinstance(prog, ast.Program):
        return
    root = prog.root
    main_files = [module.filename for module in prog.pkgs[ast.Program.MAIN_PKGPATH]]
    for pkgpath in prog.pkgs or []:
        for m in prog.pkgs[pkgpath]:
            for import_spec in m.GetImportList():
                pkgpath = import_spec.path
                if pkgpath in builtin.STANDARD_SYSTEM_MODULES:
                    continue

                if pkgpath.startswith(plugin.PLUGIN_MODULE_NAME):
                    plugin_name = pkgpath.replace(plugin.PLUGIN_MODULE_NAME, "")
                    if plugin_name not in plugin.get_plugin_names():
                        kcl_error.report_exception(
                            err_type=kcl_error.ErrType.CannotFindModule_TYPE,
                            file_msgs=[
                                kcl_error.ErrFileMsg(
                                    filename=import_spec.filename or m.filename,
                                    line_no=import_spec.line,
                                    col_no=import_spec.column,
                                    end_col_no=import_spec.end_column,
                                )
                            ],
                            arg_msg=kcl_error.CANNOT_FIND_MODULE_MSG.format(
                                import_spec.rawpath, plugin.get_plugin_root(plugin_name)
                            ),
                        )
                    continue

                if pkgpath not in prog.pkgs:
                    kcl_error.report_exception(
                        err_type=kcl_error.ErrType.CannotFindModule_TYPE,
                        file_msgs=[
                            kcl_error.ErrFileMsg(
                                filename=import_spec.filename or m.filename,
                                line_no=import_spec.line,
                                col_no=import_spec.column,
                                end_col_no=import_spec.end_column,
                            )
                        ],
                        arg_msg=kcl_error.CANNOT_FIND_MODULE_MSG.format(
                            import_spec.rawpath,
                            str(pathlib.Path(prog.root) / (pkgpath.replace(".", "/"))),
                        ),
                    )

                if os.path.isfile(f"{root}/{pkgpath.replace('.', '/')}.k"):
                    file = f"{root}/{pkgpath.replace('.', '/')}.k"
                    if file in main_files or []:
                        kcl_error.report_exception(
                            file_msgs=[
                                kcl_error.ErrFileMsg(
                                    filename=import_spec.filename,
                                    line_no=import_spec.line,
                                    col_no=import_spec.column,
                                )
                            ],
                            err_type=kcl_error.ErrType.CompileError_TYPE,
                            arg_msg=f"Cannot import {file} in the main package",
                        )


def ResolveProgram(
    program: ast.Program, config: CheckConfig = CheckConfig()
) -> ProgramScope:
    """Resolve program including the import check and the type check"""
    ResolveProgramImport(program)
    return TypeChecker(program, config).check()
