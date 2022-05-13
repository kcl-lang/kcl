# Copyright 2021 The KCL Authors. All rights reserved.
from copy import deepcopy
from dataclasses import dataclass, field
from typing import Optional, Union, List, Dict
from .decorator import KCLDecoratorObject
from .object import (
    KCLObject,
    KCLObjectType,
    KCLTrueObject,
    KCLFalseObject,
    KCLIntObject,
    KCLFloatObject,
    KCLStringObject,
    KCLBaseTypeObject,
    KCLDictObject,
    KCLSchemaObject,
    KCLNoneObject,
    KCLUndefinedObject,
    KCLTypeKind,
    KCLAnyTypeObject,
    KCLStringTypeObject,
    KCLSchemaReverseFields,
    to_kcl_obj,
)
from .function import KWArg, KCLCompiledFunctionObject

import kclvm.kcl.error as kcl_error
import kclvm.kcl.info as kcl_info
import kclvm.api.object.internal.common as common

from kclvm.internal.util import hash
from kclvm.compiler.check.check_type import (
    type_pack_and_check,
    check_type,
    has_literal_type,
)
from kclvm.kcl.ast import ast

SETTINGS_OUTPUT_KEY = "output_type"
SETTINGS_OUTPUT_STANDALONE = "STANDALONE"
SETTINGS_OUTPUT_INLINE = "INLINE"
SETTINGS_OUTPUT_IGNORE = "IGNORE"

SCHEMA_SETTINGS_ATTR_NAME = "__settings__"
SCHEMA_TYPE_ATTR_NAME = "__schema_type__"
SCHEMA_RUNTIME_TYPE_ATTR_NAME = "__runtime_schema_type__"
MAIN_MODULE_NAME = "__main__"

SCHEMA_SELF_VALUE_KEY = "$schema_self"
SCHEMA_CONFIG_VALUE_KEY = "$schema_config"
SCHEMA_CONFIG_META_KEY = "$schema_config_meta"


class RefGraph:
    """
    Reference graph
    """

    def __init__(self):
        self.adjs = {}

    def _find_node_index(self, node):
        return self.nodeset.index(node)

    def add_node_judge_cycle(self, node, another_node):
        """
        add edge into the schema inheritance graph and check if cyclic inheritance occurs in schema
        """
        if node not in self.adjs:
            self.adjs[node] = []
        if another_node not in self.adjs:
            self.adjs[another_node] = []
        self.adjs[another_node].append(node)
        return self._has_cycle()

    def _has_cycle(self):
        """
        Determine whether the schema inheritance graph is a Directed Acyclic Graph (DAG).
        The detection uses Depth First Search (DFS) algorithm for each node
        in the ergodic graph, and the time complexity is O (V + E),
        V: the total number of detected nodes,
        E: the total number of edges connected by nodes
        """
        visited = {name: 0 for name in self.adjs.keys()}

        def _dfs(name):
            visited[name] = 1
            for adj in self.adjs[name]:
                if visited[adj] == 1:
                    return True
                if visited[adj] == 0:
                    if _dfs(adj):
                        return True
                    else:
                        continue
            visited[name] = 2
            return False

        for name in visited.keys():
            if visited[name] == 0 and _dfs(name):
                return True
        return False


class SchemaTypeRefGraph(RefGraph):
    def get_sub_schemas(self, name: str) -> list:
        """Get all sub schemas by name using BFS"""
        result = []
        if not name:
            return result
        sub_schemas = self.adjs.get(name, [])
        result += sub_schemas
        for sub_schema in sub_schemas:
            result += self.get_sub_schemas(sub_schema)
        return result


class SchemaTypeFactory:
    """
    A schema_type factory used to get schema_type object.
    """

    def __init__(self):
        self._schema_types = {}

    def register(self, name: str, schema_type: "KCLSchemaTypeObject"):
        """
        Register a schema_type with a unique name.

        :param name: Name of the schema_type
        :param schema_type: The schema_type to be registered
        :return: None
        """
        self._schema_types[name] = schema_type

    def get(self, name: str):
        """
        Get and return a schema_type object.

        :param name: Name of the schema_type
        :return: A schema_type object
        """
        schema_type = self._schema_types.get(name)
        if not schema_type:
            raise Exception(f"unknown schema type '{name}'")
        return schema_type


@dataclass
class KCLSchemaIndexSignatureObject(KCLObject):
    key_name: Optional[str] = None
    key_type: str = None
    value_type: str = None
    value: KCLObject = None
    any_other: bool = None
    key_kcl_type: KCLBaseTypeObject = None
    value_kcl_type: KCLBaseTypeObject = None
    node: ast.SchemaIndexSignature = None

    def type(self) -> KCLObjectType:
        """
        Get the object type
        """
        return KCLObjectType.SCHEMA_INDEX_SIGNATURE

    def type_str(self) -> str:
        """
        Get the object type string
        """
        return "IndexSignatureType"

    def def_str(self) -> str:
        return (
            # key_name
            f"[{self.key_name + ': ' if self.key_name else ''}"
            # ...
            + ("..." if self.any_other else "")
            # key_type
            + f"{self.key_type}]: "
            # value_type
            + f"{self.value_type}"
        )


@dataclass
class KCLSchemaAttrObject(KCLObject):
    is_optional: bool = True
    is_final: bool = False
    has_default: bool = False
    attr_type: KCLBaseTypeObject = None
    attr_node: Optional[ast.AST] = None


@dataclass
class KCLSchemaTypeObject(KCLBaseTypeObject):
    name: str = None
    MEMBER_FUNCTIONS = ["instances"]  # Member function list
    name: Optional[str] = None  # Schema name
    __refs__: Optional[list] = field(default_factory=list)  # Instance reference list
    func: Optional[KCLCompiledFunctionObject] = None  # Body functions
    check_fn: Optional[KCLCompiledFunctionObject] = None  # Check function
    is_mixin: bool = False  # Mark is a schema mixin
    is_protocol: bool = False  # Mark is a schema protocol
    is_rule: bool = False  # Mark is a rule block
    is_relaxed: bool = False  # Mark is a relaxed schema
    pkgpath: str = ""  # Schema definition package path
    filename: str = ""  # Definition path location
    doc: str = ""  # Schema definition document string
    runtime_type: Optional[str] = None  # Schema runtime type file_hash + schema_name
    base: Optional["KCLSchemaTypeObject"] = None  # Base schema
    protocol: Optional["KCLSchemaTypeObject"] = None  # Protocol schema
    mixins_names: Optional[List[str]] = field(default_factory=list)
    mixins: List["KCLSchemaTypeObject"] = field(
        default_factory=list
    )  # Schema mixin list
    attrs: Optional[dict] = field(default_factory=dict)  # Schema attributes
    attr_list: Optional[list] = field(default_factory=list)  # Schema attribute order
    attr_obj_map: Dict[Union[str, int, float], Optional[KCLSchemaAttrObject]] = field(
        default_factory=dict
    )  # Schema attribute type map
    settings: Optional[dict] = field(default_factory=dict)  # Schema settings
    decorators: Optional[List[KCLDecoratorObject]] = field(
        default_factory=list
    )  # Schema decorator list
    index_signature: Optional[
        KCLSchemaIndexSignatureObject
    ] = None  # Schema Index signature
    node_ref: Optional[ast.SchemaStmt] = None

    # -----------------
    # Schema eval cache
    # -----------------

    _eval_cache = {}

    def can_add_members(self) -> bool:
        return (
            self.name.endswith("Mixin")
            or self.index_signature is not None
            or self.is_relaxed
        )

    def type(self) -> KCLObjectType:
        """
        Get the object type
        """
        return KCLObjectType.SCHEMA_TYPE

    def type_str(self) -> str:
        """
        Get the object type string
        """
        return (
            self.name
            if (self.pkgpath == "__main__" or not self.pkgpath)
            else f"{self.pkgpath}.{self.name}"
        )

    def type_str_with_pkgpath(self) -> str:
        """
        Get the object type string with pkgpath
        """
        return (
            self.name
            if (self.pkgpath == "__main__" or not self.pkgpath)
            else f"@{self.pkgpath}.{self.name}"
        )

    def type_kind(self):
        return KCLTypeKind.SchemaKind

    @property
    def value(self) -> str:
        """Return the runtime type string"""
        return self.runtime_type

    @property
    def key_type(self) -> str:
        if self.index_signature:
            return self.index_signature.key_kcl_type
        return KCLStringTypeObject()

    @property
    def value_type(self) -> str:
        if self.index_signature:
            return self.index_signature.value_kcl_type
        return KCLAnyTypeObject()

    @property
    def should_add_additional_key(self) -> bool:
        return self.is_relaxed or self.index_signature is not None

    @property
    def file_and_type(self) -> str:
        return self.filename + self.runtime_type

    @property
    def file_and_name(self) -> str:
        return self.filename + self.name

    def is_sub_schema_of(self, base: Union["KCLSchemaTypeObject", str]) -> bool:
        base_type_obj = base
        if not isinstance(base_type_obj, KCLSchemaTypeObject):
            return False
        if (
            self.runtime_type == base.runtime_type
            or self.file_and_name == base.file_and_name
        ):
            return True
        base_ref = self.base
        while base_ref and base_ref.runtime_type != base_type_obj.runtime_type:
            base_ref = base_ref.base
        return True if base_ref else False

    def get_obj_of_attr(self, attr: Union[int, str]) -> Optional[KCLSchemaAttrObject]:
        if attr in self.attr_obj_map:
            return self.attr_obj_map[attr]
        base_ref = self.base
        while base_ref and attr not in base_ref.attr_obj_map:
            base_ref = base_ref.base
        if base_ref:
            return base_ref.attr_obj_map[attr]
        return (
            self.protocol.attr_obj_map[attr]
            if self.protocol and attr in self.protocol.attr_obj_map
            else None
        )

    def get_type_of_attr(self, attr: Union[int, str]) -> Optional[KCLBaseTypeObject]:
        attr_obj = self.get_obj_of_attr(attr)
        return attr_obj.attr_type if attr_obj else None

    def get_node_of_attr(self, attr: Union[int, str]) -> Optional[ast.AST]:
        attr_obj = self.get_obj_of_attr(attr)
        return attr_obj.attr_node if attr_obj else None

    def set_type_of_attr(self, attr: Union[int, str], tpe: KCLBaseTypeObject):
        if attr in self.attr_obj_map:
            self.attr_obj_map[attr].attr_type = tpe
        else:
            self.attr_obj_map[attr] = KCLSchemaAttrObject(attr_type=tpe)

    def set_node_of_attr(self, attr: Union[int, str], ast_node: ast.AST):
        if attr in self.attr_obj_map:
            self.attr_obj_map[attr].attr_node = ast_node
        else:
            self.attr_obj_map[attr] = KCLSchemaAttrObject(attr_node=ast_node)

    def add_decorators(self, decorators: List[KCLDecoratorObject]):
        if not self.decorators:
            self.decorators = []
        self.decorators += decorators

    def set_func(self, func: KCLCompiledFunctionObject):
        assert isinstance(func, KCLCompiledFunctionObject)
        self.func = func

    def set_check_func(self, check_fn: Optional[KCLCompiledFunctionObject]):
        if check_fn:
            assert isinstance(check_fn, KCLCompiledFunctionObject)
            self.check_fn = check_fn
        else:
            self.check_fn = None

    @staticmethod
    def schema_runtime_type(tpe: str, filename=""):
        return f"runtime_type_{hash(filename)}_{tpe}"

    @staticmethod
    def new(
        name: str,
        base: Optional["KCLSchemaTypeObject"] = None,
        protocol: Optional["KCLSchemaTypeObject"] = None,
        filename: str = "",
        is_relaxed: bool = False,
        is_mixin: bool = False,
        pkgpath: str = "",
        attr_list: list = None,
        index_signature: Optional[KCLSchemaIndexSignatureObject] = None,
        vm=None,
    ):
        if not name:
            kcl_error.report_exception(
                err_type=kcl_error.ErrType.EvaluationError_TYPE,
                arg_msg="schema name can't be None",
            )
        runtime_type = KCLSchemaTypeObject.schema_runtime_type(name, pkgpath)
        settings = {
            SETTINGS_OUTPUT_KEY: SETTINGS_OUTPUT_INLINE
            if not kcl_info.isprivate_field(name)
            else SETTINGS_OUTPUT_IGNORE,
            KCLSchemaReverseFields.NAME: name,
            KCLSchemaReverseFields.TYPE: f"{pkgpath}.{name}",
            KCLSchemaReverseFields.PKG_PATH: pkgpath,
        }
        obj = KCLSchemaTypeObject(
            pkgpath=pkgpath,
            name=name,
            base=base,
            protocol=protocol,
            settings=settings,
            filename=filename,
            is_relaxed=is_relaxed,
            is_mixin=is_mixin,
            runtime_type=runtime_type,
            attrs={attr: KCLUndefinedObject.instance() for attr in attr_list},
            attr_list=attr_list,
            index_signature=index_signature,
        )
        return obj

    def update_mixins(self, vm=None):
        """Get mixins by name"""
        if not self.mixins and self.mixins_names:
            self.mixins = []
            for mixin_name in self.mixins_names:
                if "." in mixin_name:  # pkg.Schema
                    schema_type_obj = vm.find_schema_type(mixin_name)
                    if schema_type_obj and isinstance(
                        schema_type_obj, KCLSchemaTypeObject
                    ):
                        self.mixins.append(schema_type_obj)
                    else:
                        kcl_error.report_exception(
                            err_type=kcl_error.ErrType.EvaluationError_TYPE,
                            arg_msg="name '{}' is not defined".format(mixin_name),
                        )
                else:
                    if mixin_name not in vm.frames[0].globals:
                        kcl_error.report_exception(
                            err_type=kcl_error.ErrType.EvaluationError_TYPE,
                            arg_msg="name '{}' is not defined".format(mixin_name),
                        )
                    schema_type_obj = vm.frames[0].globals[mixin_name]
                    self.mixins.append(schema_type_obj)

    def has_base(self):
        return self.base and isinstance(self.base, KCLSchemaTypeObject)

    def new_empty_instance(self) -> KCLSchemaObject:
        return KCLSchemaObject(
            attrs={SCHEMA_SETTINGS_ATTR_NAME: to_kcl_obj(self.settings)},
            pkgpath=self.pkgpath,
            name=self.name,
            runtime_type=self.runtime_type,
        )

    def new_instance(
        self,
        config: Union[dict, KCLDictObject],
        config_meta: dict,
        args: List[KCLObject],
        kwargs: List[KWArg],
        vm,
    ):
        from kclvm.vm.runtime.evaluator import SchemaEvalContext

        context = SchemaEvalContext.build_from_vm(
            vm=vm,
            type_obj=self,
            schema_obj=self.new_empty_instance(),
            config=config,
            config_meta=config_meta,
            args=args,
            kwargs=kwargs,
        )
        # Save origin eval context
        org_eval_ctx = vm.lazy_eval_ctx
        vm.lazy_eval_ctx = context
        # Reset the eval status before the evaluation
        context.eval_reset()
        # Get all schema attribute value place holders using the schema type object and the cache
        if self.file_and_type in self._eval_cache:
            context.place_holder_map = self._eval_cache[self.file_and_type]
        else:
            context.get_all_place_holders()
            self._eval_cache[self.file_and_type] = context.place_holder_map
        # New a schema instance using the type object
        context.schema_obj = self._new_instance(
            config,
            config_meta,
            args,
            kwargs,
            vm,
            # Put the schema instance reference
            inst=context.schema_obj,
        )
        # Reset the eval status after the evaluation
        context.eval_reset()
        # Reload origin eval context
        vm.lazy_eval_ctx = org_eval_ctx
        # Return the schema instance
        context.schema_obj.config_keys = (
            set(config.value.keys())
            if isinstance(config, KCLDictObject)
            else set(config.keys())
        )
        return context.schema_obj

    def _new_instance(
        self,
        config: Union[dict, KCLDictObject],
        config_meta: dict,
        args: List[KCLObject],
        kwargs: List[KWArg],
        vm,
        inst: KCLSchemaObject = None,
        is_sub_schema: bool = False,
    ) -> KCLSchemaObject:
        self.do_args_type_check(args, kwargs, config_meta, vm)
        inst = inst or self.new_empty_instance()
        inst.instance_pkgpath = vm.ctx.pkgpath
        for decorator in self.decorators:
            inst.add_decorator(self.name, decorator=decorator)
        if self.base and isinstance(self.base, KCLSchemaTypeObject):
            inst = self.base._new_instance(
                config, config_meta, [], [], vm, inst, is_sub_schema=True
            )
        # Record all schema attributes
        inst.union_with(self.attrs, should_idempotent_check=False)
        for mixin in self.mixins:
            inst.union_with(mixin.attrs, should_idempotent_check=False)
        # Record the schema name, runtime_type and relaxed
        inst.update_info(self.name, self.runtime_type, self.is_relaxed)
        vm.push_frame_using_callable(
            self.pkgpath,
            self.func,
            (args if args else []) + [config_meta, config, inst],
            kwargs,
            args_len=len(args),
        )
        # Run the schema compiled function body
        vm.run(run_current=True, ignore_nop=True)

        if SCHEMA_SETTINGS_ATTR_NAME not in inst:
            inst.update({SCHEMA_SETTINGS_ATTR_NAME: self.settings})
        if SCHEMA_SETTINGS_ATTR_NAME in config:
            inst.update(
                {SCHEMA_SETTINGS_ATTR_NAME: config.get(SCHEMA_SETTINGS_ATTR_NAME)}
            )

        # Add settings attr
        if not self.attrs:
            self.attrs = {}
        self.attrs[SCHEMA_SETTINGS_ATTR_NAME] = (
            inst.get(SCHEMA_SETTINGS_ATTR_NAME) or self.settings
        )

        # Do relaxed schema check and config patch
        relaxed_keys = self.do_relaxed_check(
            inst, config_meta, config, is_sub_schema, vm
        )

        # Record the schema name, runtime_type and relaxed
        inst.update_info(self.name, self.runtime_type, self.is_relaxed)

        self.update_mixins(vm=vm)

        # Do all mixins expand execution after schema context
        if self.mixins:
            for mixin in self.mixins:
                inst = mixin._new_instance(
                    config, config_meta, [], [], vm, inst, is_sub_schema=True
                )

        # Record the schema name, runtime_type and relaxed
        inst.update_info(self.name, self.runtime_type, self.is_relaxed)

        # Record schema instance
        if not self.__refs__:
            self.__refs__ = []
        self.__refs__.append(inst)

        # Deal schema stmt queue
        if not is_sub_schema and inst.stmt_buffer():
            buffers = inst.stmt_buffer()
            func = KCLCompiledFunctionObject(
                name=self.func.name,
                params=self.func.params,
                names=self.func.names,
                constants=self.func.constants,
            )
            relaxed = inst.is_relaxed
            for buffer in buffers:
                is_relaxed, pkg_path, codes = buffer
                func.instructions = codes
                inst.is_relaxed = is_relaxed
                vm.push_frame_using_callable(
                    pkg_path,
                    func,
                    (args if args else []) + [config_meta, config, inst],
                    kwargs,
                    args_len=len(args),
                )
                # Run the schema compiled function body
                vm.run(run_current=True, ignore_nop=True)
            inst.is_relaxed = relaxed
        # Run all decorators
        inst.run_all_decorators()
        # Do all checks
        if not is_sub_schema:
            inst.operation_map = {
                KCLSchemaReverseFields.SETTINGS: ast.ConfigEntryOperation.OVERRIDE
            }
            inst.update_attr_op_using_obj(config)
            inst.check_optional_attrs()
            self.do_check(inst, config_meta, config, relaxed_keys, vm)
        # Return the schema object
        return inst

    def do_args_type_check(
        self,
        args: List[KCLObject],
        kwargs: List[KWArg],
        config_meta: dict,
        vm=None,
    ):
        """Check args type"""

        def check_arg_type(arg_name: str, value: KCLObject, expected_type: str):
            checked, value_tpe = check_type(value, expected_type, vm)
            if not checked:
                if has_literal_type([expected_type]):
                    if isinstance(
                        value,
                        (
                            KCLNoneObject,
                            KCLTrueObject,
                            KCLFalseObject,
                            KCLIntObject,
                            KCLFloatObject,
                        ),
                    ):
                        value_tpe = f"{value_tpe}({value.value})"
                    elif isinstance(value, KCLStringObject):
                        value_tpe = f'{value_tpe}("{value.value}")'

                conf_filename, conf_line, conf_column = (
                    config_meta.get("$filename"),
                    config_meta.get("$lineno"),
                    config_meta.get("$columnno"),
                )
                kcl_error.report_exception(
                    err_type=kcl_error.ErrType.TypeError_Runtime_TYPE,
                    file_msgs=[
                        kcl_error.ErrFileMsg(
                            filename=conf_filename,
                            line_no=conf_line,
                            col_no=conf_column,
                        )
                    ],
                    arg_msg='argument "{}" expect {}, got {}'.format(
                        arg_name,
                        common.get_tpes_str([expected_type]).replace("@", ""),
                        common.get_class_name(value_tpe),
                    ),
                )

        if self.func.params:
            for i, value in enumerate(args or []):
                arg_name = self.func.params[i].name
                expected_type = self.func.params[i].type_annotation
                check_arg_type(arg_name, value, expected_type)

            for kwarg in kwargs or []:
                arg_name = kwarg.name.value
                value = kwarg.value
                if arg_name not in [p.name for p in self.func.params]:
                    kcl_error.report_exception(
                        err_type=kcl_error.ErrType.EvaluationError_TYPE,
                        arg_msg=f"schema arguments got an unexpected keyword argument '{arg_name}'",
                    )
                expected_types = [
                    p.type_annotation for p in self.func.params if arg_name == p.name
                ]
                expected_type = expected_types[0] if expected_types else None
                check_arg_type(arg_name, value, expected_type)

    def do_relaxed_check(
        self,
        inst: KCLSchemaObject,
        config_meta: dict,
        config: Union[dict, KCLDictObject],
        is_sub_schema: bool,
        vm,
    ) -> List[str]:
        """Do relaxed schema check and config patch"""
        relaxed_keys = []
        if not is_sub_schema:
            config_native = (
                config.value if isinstance(config, KCLDictObject) else config
            )
            config_meta_native = config_meta
            relaxed_keys = [
                key for key in config_native if key not in inst.value.keys()
            ]
            if self.protocol:
                relaxed_keys = [
                    key for key in relaxed_keys if key not in self.protocol.attr_list
                ]
            if self.is_relaxed or self.index_signature:
                filename = vm.get_filename()
                if self.index_signature and not self.index_signature.any_other:
                    for key in inst.value.keys():
                        if key != SCHEMA_SETTINGS_ATTR_NAME:
                            value = inst.get(key)
                            checked, _ = check_type(
                                value,
                                self.index_signature.value_type,
                                vm=vm,
                            )
                            if not checked:
                                kcl_error.report_exception(
                                    err_type=kcl_error.ErrType.EvaluationError_TYPE,
                                    file_msgs=[kcl_error.ErrFileMsg(filename=filename)],
                                    arg_msg=f"the type '{value.type_str()}' of schema attribute '{key}' "
                                    f"does not meet the index signature definition {self.index_signature.def_str()}",
                                )
                for key in relaxed_keys:
                    lineno, columnno = None, None
                    if key in config_meta_native:
                        lineno, columnno = (
                            config_meta_native[key].get("lineno"),
                            config_meta_native[key].get("columnno"),
                        )
                    value = config.get(key)
                    if self.index_signature and self.index_signature.value_type:
                        types = [self.index_signature.value_type]
                        from kclvm.vm.runtime.evaluator import union

                        value = type_pack_and_check(
                            union(
                                deepcopy(self.index_signature.value),
                                value,
                                should_idempotent_check=True,
                                vm=vm,
                            ),
                            types,
                            filename,
                            lineno,
                            columnno,
                            vm=vm,
                            config_meta=config_meta,
                        )
                    inst.update({key: value})

            elif relaxed_keys:
                lineno, columnno = None, None
                if relaxed_keys[0] in config_meta_native:
                    lineno, columnno = (
                        config_meta_native[relaxed_keys[0]].get("lineno"),
                        config_meta_native[relaxed_keys[0]].get("columnno"),
                    )
                kcl_error.report_exception(
                    err_type=kcl_error.ErrType.CannotAddMembers_Runtime_TYPE,
                    file_msgs=[
                        kcl_error.ErrFileMsg(
                            filename=vm.get_filename(), line_no=lineno, col_no=columnno
                        )
                    ],
                    arg_msg=kcl_error.CANNOT_ADD_MEMBERS_MSG.format(
                        ",".join([str(k) for k in relaxed_keys]), self.name
                    ),
                )
        return relaxed_keys

    def do_check(
        self,
        inst: KCLSchemaObject,
        config_meta: dict,
        config: Union[dict, KCLDictObject],
        relaxed_keys: List[str],
        vm,
    ):
        assert inst, f"{inst}"
        assert vm

        def call_check_fn(local_name: str = None, local_value: KCLObject = None):
            if self.check_fn:
                vm.push_frame_using_callable(
                    self.pkgpath, self.check_fn, [config_meta, config, inst], []
                )
                if local_name and local_value:
                    vm.update_local(local_name, local_value)
                vm.run(run_current=True, ignore_nop=True)

        # check base
        if self.base and isinstance(self.base, KCLSchemaTypeObject):
            self.base.do_check(inst, config_meta, config, relaxed_keys, vm)

        # check mixin
        for mixin in self.mixins or []:
            mixin.do_check(inst, config_meta, config, relaxed_keys, vm)

        # check self
        if self.index_signature and self.index_signature.key_name and relaxed_keys:
            # For loop index signature attributes
            for key in relaxed_keys:
                call_check_fn(self.index_signature.key_name, to_kcl_obj(key))
        else:
            call_check_fn()

    # Member Functions

    def instances(self, main_pkg: bool = True):
        """Get all schema instances of self type and sub types"""
        if not self.__refs__:
            self.__refs__ = []
        return deepcopy(
            [
                inst
                for inst in self.__refs__
                if inst.instance_pkgpath == MAIN_MODULE_NAME
            ]
            if main_pkg
            else self.__refs__
        )

    def get_member_method(self, name: str):
        from .function import KCLMemberFunctionObject

        if not name:
            kcl_error.report_exception(
                err_type=kcl_error.ErrType.EvaluationError_TYPE,
                arg_msg="kcl string object member name can't be empty or None",
            )
        if name not in self.MEMBER_FUNCTIONS:
            kcl_error.report_exception(
                err_type=kcl_error.ErrType.AttributeError_TYPE,
                arg_msg=f"attribute '{name}' not found",
            )
        return KCLMemberFunctionObject(obj=self, name=name)

    def call_member_method(self, name: str, *args, **kwargs):
        if not hasattr(self, name) and name not in self.MEMBER_FUNCTIONS:
            kcl_error.report_exception(
                err_type=kcl_error.ErrType.AttributeError_TYPE,
                arg_msg=f"attribute '{name}' not found",
            )
        return getattr(self, name).__call__(*args, **kwargs)


@dataclass
class KCLSchemaDefTypeObject(KCLBaseTypeObject):
    """Schema definition type denotes the schema definition type used in normal expressions.

    - `Person` of `data = Person.instances()` is a schema def type.
    - `person` of `person = Person {}` is a schema type.
    """

    schema_type: KCLSchemaTypeObject

    def type_str(self) -> str:
        """Get the object type"""
        return self.schema_type.type_str() if self.schema_type else super().type_str()

    def type_kind(self) -> int:
        """Get the"""
        return KCLTypeKind.SchemaDefKind
