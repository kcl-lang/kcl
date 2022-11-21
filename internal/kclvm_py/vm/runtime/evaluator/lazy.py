# Copyright 2021 The KCL Authors. All rights reserved.

import inspect

from dataclasses import dataclass, field
from typing import List, Dict, Union, Optional, Callable, TypeVar
from enum import IntEnum

import kclvm.kcl.error as kcl_error
import kclvm.api.object as obj
import kclvm.vm.code as vm_code
import kclvm.compiler.extension.builtin as builtin


# --------------------
# Type alias
# --------------------

SchemaKeyNativeType = Union[str, int, float]
SchemaKeyObjectType = Union[obj.KCLStringObject, obj.KCLIntObject, obj.KCLFloatObject]
SchemaKeyType = Union[SchemaKeyNativeType, SchemaKeyObjectType]
VirtualMachine = TypeVar("vm.VirtualMachine")


# ------------------------------------
# Schema attribute value place holder
# ------------------------------------


class ValuePlaceHolderPriority(IntEnum):
    """
    Value place holder override priority (Ascending)
    ------------------------------------------------
    1. base_default (default value of the base class schema attribute)
    2. base_templating (base schema writing general expression context)
    3. base_mixin (base class schema attribute)
    4. sub_default (the default value of the schema attribute of the subclass)
    5. sub_templating (subclass schema writing general expression context)
    6. sub_mixin (context of subclass schema)
    7. config (schema instantiation configuration value)
    """

    BASE_DEFAULT = 1
    BASE_TEMPLATING = 2
    BASE_MIXIN = 3
    SUB_DEFAULT = 4
    SUB_TEMPLATING = 5
    SUB_MIXIN = 6
    CONFIG = 7


@dataclass
class ValuePlaceHolder:
    """The real value of the schema attribute can be regarded
    as a placeholder for a non-literal expression before it is calculated

    Please note that all optional attribute checks, relaxed attr append,
    schema index signature and decorator execution and check block are executed
    after the calculation of the above process is completed.
    """

    name: str  # Attribute name
    priority: ValuePlaceHolderPriority
    types: List[str] = field(default_factory=list)  # Attribute type annotation list
    value: obj.KCLObject = None  # Actual value
    config_value: obj.KCLObject = None  # Config value
    codes: List[vm_code.SchemaBodyOpcodeFactory] = field(
        default_factory=list
    )  # VM eval byte codes


# --------------------
# Cache definition
# --------------------


class ValueCache:
    """Key-value pair cache
    - The key is a valid key type including `str`, `int` and `float`
    - The value is a KCL object
    """

    def __init__(self):
        self._cache: Dict[SchemaKeyNativeType, obj.KCLObject] = {}

    def __getitem__(self, key):
        return self.get(key)

    def __contains__(self, key):
        return key in self._cache

    def clear(self):
        self._cache.clear()

    def get(
        self, key: SchemaKeyNativeType, eval_func: Callable = None, *args, **kwargs
    ) -> Optional[obj.KCLObject]:
        """Get the value from the cache. When the parameter `eval_func` is
        provided and there is no such value in the cache, the value is calculated
        by calling `eval_func`
        """
        if key not in self._cache and eval_func and inspect.isfunction(eval_func):
            self._cache[key] = eval_func(*args, **kwargs)
        return self._cache.get(key)

    def set(self, key: SchemaKeyNativeType, value: obj.KCLObject):
        if not key:
            raise ValueError(f"Invalid key {key}")
        if not value or not isinstance(value, obj.KCLObject):
            raise ValueError(f"Invalid kcl object {value}")
        self._cache[key] = value


# --------------------
# Backtracking
# --------------------


@dataclass
class Backtracking:
    """Backtracking calculation set

    `__enter__` and `__exit__` function can be used as follows:

        backtracking = Backtracking()
        with backtracking.catch(name):
            assert Backtracking.tracking_level(name) == 1
            with backtracking.catch(name):
                assert Backtracking.tracking_level(name) == 2
            assert Backtracking.tracking_level(name) == 1
        assert Backtracking.tracking_level(name) == 0
    """

    _set: dict = field(default_factory=dict)
    _key: Optional[str] = None

    def __enter__(self):
        if self._key not in self._set:
            self._set[self._key] = 1
        else:
            self._set[self._key] += 1

    def __exit__(self, e_t, e_v, t_b):
        if self._key in self._set:
            self._set[self._key] -= 1
            if self._set[self._key] == 0:
                del self._set[self._key]

    def catch(self, key: str):
        self._key = key
        return self

    def is_backtracking(self, key: str):
        return key in self._set and self.tracking_level(key) > 0

    def tracking_level(self, key: str):
        return self._set.get(key, 0)

    def reset(self):
        self._set.clear()


# --------------------
# Schema eval context
# --------------------


@dataclass
class SchemaEvalContext:
    """Schema irrelevant order calculation context

    TODO: Need to be combined with configuration merging technology and
    move it into the compiler stage.
    """

    type_obj: obj.KCLSchemaTypeObject
    schema_obj: obj.KCLSchemaObject
    config: obj.KCLDictObject
    config_meta: dict
    args: List[obj.KCLObject]
    kwargs: List[obj.KWArg]
    vm: VirtualMachine
    cache: ValueCache = field(default_factory=ValueCache)

    place_holder_map: Dict[str, ValuePlaceHolder] = field(default_factory=lambda: None)
    backtracking: Backtracking = field(default_factory=Backtracking)

    def eval_reset(self):
        """Eval status reset to prevent reference interference"""
        self.cache.clear()
        self.backtracking.reset()

    # ----------------------------
    # Code to value place holders
    # ----------------------------

    def code_to_place_holder(
        self,
        code: vm_code.SchemaBodyOpcodeFactory,
        name: str,
        priority: ValuePlaceHolderPriority,
    ) -> ValuePlaceHolder:
        """Covert schema body code to value place holder"""
        assert (
            name
            and isinstance(name, str)
            and code
            and isinstance(code, vm_code.SchemaBodyOpcodeFactory)
        )
        place_holder = ValuePlaceHolder(
            name=name,
            priority=priority,
            types=self.schema_obj.get_attr_type(name),
            codes=[code],
            config_value=self.config.get(name),
        )
        return place_holder

    def get_all_place_holders(self) -> Dict[str, ValuePlaceHolder]:
        """Get all schema attribute value place holders"""
        self.place_holder_map = self.get_place_holders_by_type_obj(
            self.type_obj, {}, ValuePlaceHolderPriority.SUB_DEFAULT
        )
        return self.place_holder_map

    def get_place_holders_by_type_obj(
        self,
        type_obj: obj.KCLSchemaTypeObject,
        map_ref: dict,
        priority: ValuePlaceHolderPriority,
    ) -> Dict[str, ValuePlaceHolder]:
        """Get schema place holders using the schema type object"""
        if not type_obj or not isinstance(type_obj, obj.KCLSchemaTypeObject):
            return {}
        # Get the base schema place holders
        if type_obj.base:
            self.get_place_holders_by_type_obj(
                type_obj.base, map_ref, ValuePlaceHolderPriority.BASE_DEFAULT
            )
        code_factory = vm_code.SchemaBodyOpcodeFactory.build_from_codes(
            type_obj.func.instructions,
            type_obj.pkgpath,
            type_obj.name,
        )
        # Split opcodes into key-value pair `{attr_name}-{opcode_list}`
        codes = code_factory.split_to_schema_attr_codes()
        assert len(codes) == len(
            type_obj.attr_list
        ), f"{len(codes)} != {len(type_obj.attr_list)}"
        for i, code in enumerate(codes):
            place_holder = self.code_to_place_holder(
                code, type_obj.attr_list[i], priority + 1
            )
            if (
                place_holder
                and place_holder.name in map_ref
                and place_holder.priority >= map_ref[place_holder.name].priority
            ):
                # Schema attribute multi-override
                map_ref[place_holder.name].codes.extend(place_holder.codes)
            else:
                # First default value place holder
                map_ref[place_holder.name] = place_holder
        # Get the mixin place holders
        for mixin in type_obj.mixins or []:
            self.get_place_holders_by_type_obj(mixin, map_ref, priority + 2)
        return map_ref

    # ----------------------------
    # Back track
    # ----------------------------

    def back_track(self, name: str, place_holder: ValuePlaceHolder):
        if not name or not place_holder:
            return
        try:
            if self.backtracking.is_backtracking(name):
                pass
                """
                # Please note that KCL variable reference is not considered a self circular reference,
                # and the value may be overwritten such as `a = 1; a += 2` will be treated as `a = 1 + 2`
                raise kcl_error.KCLRuntimeError(
                    "RecursionError", None, None, None, f"Attribute '{name}' reference cycle"
                )
                """
            with self.backtracking.catch(name):
                # Number of variable backtracking
                level = self.backtracking.tracking_level(name)
                # Get the place holder code. Please note the negative index `-level`
                # because the later value place holders are placed at the end of the array
                code: vm_code.SchemaBodyOpcodeFactory = place_holder.codes[-level]
                # Run schema attribute place holder code
                self.vm_run_schema_code(code)
            # When the traceback ends, save the value to the cache
            self.cache.set(name, self.schema_obj.get(name))
        except IndexError:
            kcl_error.report_exception(
                err_type=kcl_error.ErrType.RecursionError_TYPE,
                arg_msg=f"Attribute '{name}' reference cycle",
            )

    # -------------------------------
    # Schema Attribute Getter/Setter
    # -------------------------------

    def set_value(self, name: SchemaKeyNativeType, value: obj.KCLObject):
        self.schema_obj.update({name: value})
        # If the attribute has only one place holder, its value is also the cache value
        if name in self.place_holder_map:
            if len(self.place_holder_map[name].codes) <= 1:
                self.cache.set(name, value)
            elif (
                self.place_holder_map[name].codes[-1].begin_index
                < self.vm.ctx.isp
                < self.place_holder_map[name].codes[-1].end_index
                and self.place_holder_map[name].codes[-1].schema_name
                == self.vm.ctx.name
            ):
                self.cache.set(name, value)

    def get_value(self, name: SchemaKeyNativeType) -> Optional[obj.KCLObject]:
        if not name:
            raise ValueError(f"Invalid name: {name}")
        # Deal in-place modify and return it self immediately
        if self.is_target_attr(name) and not self.backtracking.is_backtracking(name):
            return self.schema_obj.get(name)

        if name in self.cache:
            return self.cache[name]
        elif name in self.place_holder_map:
            self.back_track(name, self.place_holder_map.get(name))

        # 1. Load from schema self
        if name in self.schema_obj:
            value = self.schema_obj.get(name)
            return value

        # 2. Load from frame locals such as loop variables and schema args
        value = self.get_from_frame_locals(name)
        if value is not None:
            return value

        # 3. Load from globals
        value = self.get_from_frame_globals(name)
        if value is not None:
            return value

        # 4. Load from builtin
        built_obj_list = builtin.get_builtin_func_objects()
        for value in built_obj_list:
            if value.name == name:
                return value

        # 5. Load from pkg because the name may be a package variable that starts with `@` character
        pkgpath = name[1:] if name.startswith("@") else name
        if pkgpath in self.vm.state.modules:
            value = self.vm.state.modules[pkgpath]
            return value
        kcl_error.report_exception(
            err_type=kcl_error.ErrType.EvaluationError_TYPE,
            arg_msg=f"'{name}' is not defined",
        )

    def is_target_attr(self, key: str) -> bool:
        if key not in self.place_holder_map:
            return False
        place_holders = self.place_holder_map.get(key)
        for code in place_holders.codes:
            if code.schema_name != self.vm.ctx.name:
                continue
            if code.begin_index <= self.vm.ctx.isp <= code.end_index:
                return True
        return False

    # -------------------------------------
    # VM schema body code runner
    # -------------------------------------

    def vm_run_schema_code(self, code: vm_code.SchemaBodyOpcodeFactory):
        if not code or not isinstance(code, vm_code.SchemaBodyOpcodeFactory):
            return
        codes_run: list = code.to_run_code_list()
        func = obj.KCLCompiledFunctionObject(
            name=code.schema_name,
            params=self.type_obj.func.params,
            names=self.type_obj.func.names,
            constants=self.type_obj.func.constants,
        )
        func.instructions = codes_run
        self.vm.push_frame_using_callable(
            code.pkgpath,
            func,
            [*self.args, self.config_meta, self.config, self.schema_obj]
            if self.args
            else [self.config_meta, self.config, self.schema_obj],
            self.kwargs,
            args_len=len(self.args),
        )
        # Run the schema compiled function body
        self.vm.run(run_current=True, ignore_nop=True)

    # -------------------------------------
    # VM frame locals/globals
    # -------------------------------------

    def get_from_frame_locals(self, key: str) -> Optional[obj.KCLDictObject]:
        """Get kcl object from vm frame locals dict"""
        if self.key_is_in_frame_locals(key):
            return self.vm.ctx.locals[key]
        index = self.vm.frame_index
        # Search the local variables from the inside to the outside schema
        while index >= self.vm.frame_index:
            index -= 1
            if key in self.vm.frames[index].locals:
                return self.vm.frames[index].locals[key]
        return None

    def get_from_frame_globals(self, key: str) -> Optional[obj.KCLDictObject]:
        """Get kcl object from vm frame locals dict"""
        return self.vm.ctx.globals[key] if self.key_is_in_frame_globals(key) else None

    def key_is_in_frame_locals(self, key: str) -> bool:
        return key and isinstance(key, str) and key in self.vm.ctx.locals

    def key_is_in_frame_globals(self, key: str) -> bool:
        return key and isinstance(key, str) and key in self.vm.ctx.globals

    # ---------------
    # Static methods
    # ---------------

    @staticmethod
    def build_from_vm(
        vm: VirtualMachine,
        type_obj: obj.KCLSchemaTypeObject,
        schema_obj: obj.KCLSchemaObject,
        config: obj.KCLDictObject,
        config_meta: dict,
        args: List[obj.KCLObject],
        kwargs: List[obj.KWArg],
    ) -> "SchemaEvalContext":
        if not type_obj or not isinstance(type_obj, obj.KCLSchemaTypeObject):
            raise ValueError(f"Invalid kcl type object {type_obj}")
        context = SchemaEvalContext(
            type_obj=type_obj,
            schema_obj=schema_obj
            or obj.KCLSchemaObject(
                attrs={
                    obj.SCHEMA_SETTINGS_ATTR_NAME: obj.to_kcl_obj(type_obj.settings)
                },
                name=type_obj.name,
                runtime_type=type_obj.runtime_type,
            ),
            config=config,
            config_meta=config_meta,
            args=args,
            kwargs=kwargs,
            vm=vm,
            cache=ValueCache(),
        )
        return context
