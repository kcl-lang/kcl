"""The `vm` file mainly contains the function `Run` which is used
# Copyright 2021 The KCL Authors. All rights reserved.
to execute the KCL bytecode obtained by the compiler module into
KCL result used to generate YAML/JSON data.
The KCL virtual machine receives a set of bytecode and uses its
own stack structure to store variables and the calculation results
of the variables, which includes `Frame` for context switching and
`VMState` for storing the import cache and executes the program
according to the corresponding opcode sequence and its operands.
When all the operation codes are executed, the entire KCL program
is also executed.
:copyright: Copyright 2020 The KCL Authors. All rights reserved.
"""

import pathlib
import dataclasses
import typing

import kclvm.kcl.error as kcl_error
import kclvm.config
import kclvm.compiler.extension.builtin
import kclvm.compiler.extension.plugin
import kclvm.compiler.check.check_type

from kclvm.api.object import (
    KCLObject,
    KCLProgram,
    KCLResult,
    KCLCompiledFunctionObject,
    KCLModuleObject,
    KCLSchemaTypeObject,
    KWArg,
)

from .code import Opcode, VM_OP_ACTIONS


@dataclasses.dataclass
class Frame:
    isp: int = 0
    name: str = None
    filename: str = None
    colno: int = 0
    lineno: int = 0
    pkgpath: str = None
    locals: dict = None
    globals: dict = None  # Global symbol and KCL object reference
    free_vars: list = None  # Free symbols
    codes: typing.List[int] = None

    def update_info(self, filename: str, lineno: int, colno: int):
        self.filename, self.lineno, self.colno = filename, lineno, colno

    def get_info(self) -> typing.Tuple[str, int, int]:
        return self.filename, self.lineno, self.colno


class VMState:
    def __init__(self):
        self.modules: typing.Dict[str, KCLModuleObject] = {}  # {pkgpath:module}
        self.globals_table: typing.Dict[
            str, typing.Dict[str, KCLObject]
        ] = {}  # {pkgpath:globals}


class VirtualMachine:
    """KCL Virtual Machine"""

    pkgpath_stack: typing.List[str] = []

    @staticmethod
    def RunApp(app: KCLProgram, *, pkg: str = None) -> KCLResult:
        # Reset cache
        KCLSchemaTypeObject._eval_cache = {}
        VirtualMachine.pkgpath_stack = []
        return VirtualMachine(app).Run(pkg=pkg)

    def __init__(self, app: KCLProgram, state: VMState = None):
        super().__init__()

        self.app: KCLProgram = app
        self.state: VMState = state or VMState()

        self.names: typing.List[str] = []
        self.constants: typing.List[KCLObject] = []
        self.stack: typing.List[KCLObject] = []
        self.last_obj: typing.Optional[KCLObject] = None
        self.ctx: typing.Optional[Frame] = None
        self.frames: typing.List[Frame] = []
        self.frame_index = 1
        self.last_popped_frame = None
        self.sp: int = 0  # Stack Pointer

        self.cur_run_pkg: str = ""

        self.all_schema_types: typing.Dict[
            str, KCLSchemaTypeObject
        ] = {}  # [f"{pkgpath}.{name}": obj]

        self.lazy_eval_ctx = None

        self._reset(app.main)

    def Run(self, *, pkg: str = None) -> KCLResult:
        pkgpath = pkg if pkg else self.app.main
        if pkgpath in self.pkgpath_stack:
            kcl_error.report_exception(
                err_type=kcl_error.ErrType.RecursiveLoad_TYPE,
                arg_msg=kcl_error.RECURSIVE_LOADING_MODULE_MSG.format(
                    pkgpath, ", ".join(self.pkgpath_stack)
                ),
            )
        if pkg:
            self.pkgpath_stack.append(pkgpath)
        self._reset(pkgpath)
        return KCLResult(self.run(), self.get_filename())

    def define_schema_type(self, absname: str, schema_type: KCLSchemaTypeObject):
        if absname.startswith("@"):
            absname = absname[1:]
        self.all_schema_types[absname] = schema_type

    def find_schema_type(self, absname: str) -> typing.Optional[KCLSchemaTypeObject]:
        if absname.startswith("@"):
            absname = absname[1:]
        return self.all_schema_types.get(absname)

    def import_name(self, _code: int, arg: int) -> typing.Optional[KCLModuleObject]:
        self.pop()

        pkgpath = self.names[arg]
        asname = self.names[arg + 1]

        assert pkgpath and asname

        if pkgpath.startswith("@"):
            pkgpath = pkgpath[1:]
        if asname.startswith("@"):
            asname = asname[1:]

        if pkgpath in self.state.modules:
            # Read from module cache
            module = self.state.modules[pkgpath]

            self.update_local(asname, module)
            self.update_global(f"@{pkgpath}", module)
            self.push(module)

            return module

        if pkgpath in kclvm.compiler.extension.builtin.STANDARD_SYSTEM_MODULES:
            module = KCLModuleObject(
                name=pkgpath,
                asname=asname,
                value=kclvm.compiler.extension.builtin.get_system_module_func_objects(
                    pkgpath
                ),
            )

            self.update_local(asname, module)
            self.update_global(f"@{pkgpath}", module)
            self.push(module)

            # Module cache
            self.state.modules[pkgpath] = module
            self.state.globals_table[pkgpath] = {}

            return module

        if pkgpath.startswith(kclvm.compiler.extension.plugin.PLUGIN_MODULE_NAME):
            module = KCLModuleObject(
                name=pkgpath.replace(
                    kclvm.compiler.extension.plugin.PLUGIN_MODULE_NAME, ""
                ),
                asname=asname,
                value=kclvm.compiler.extension.plugin.get_plugin_func_objects(pkgpath),
            )

            self.update_local(asname, module)
            self.update_global(f"@{pkgpath}", module)
            self.push(module)

            self.state.modules[pkgpath] = module
            self.state.globals_table[pkgpath] = {}

            return module

        if pkgpath in self.app.pkgs:
            pkg_vm = VirtualMachine(self.app, self.state)
            pkg_vm.all_schema_types = self.all_schema_types

            result = pkg_vm.Run(pkg=pkgpath)

            module = KCLModuleObject(name=pkgpath, asname=asname, value=result.m)

            self.update_local(asname, module)
            self.update_global(f"@{pkgpath}", module)
            self.push(module)

            self.state.modules[pkgpath] = module
            self.state.globals_table[pkgpath] = result.m

            return module
        kcl_error.report_exception(
            err_type=kcl_error.ErrType.CannotFindModule_TYPE,
            arg_msg=kcl_error.CANNOT_FIND_MODULE_MSG.format(
                pkgpath,
                str(
                    pathlib.Path(self.get_filename()).parent
                    / (pkgpath.replace(".", "/"))
                ),
            ),
        )

    def _reset(self, pkg: str):
        assert pkg in self.app.pkgs

        self.cur_run_pkg = pkg
        bytecode = self.app.pkgs[pkg]

        self.names: typing.List[str] = bytecode.names
        self.constants: typing.List[KCLObject] = bytecode.constants
        self.stack: typing.List[KCLObject] = []
        self.last_obj: typing.Optional[KCLObject] = None
        self.ctx: typing.Optional[Frame] = None
        self.frames: typing.List[Frame] = [
            Frame(
                codes=bytecode.instructions,
                pkgpath=pkg,
                locals={},
                globals={},
                free_vars=[],
            )
        ]
        self.frame_index = 1
        self.last_popped_frame = None
        self.sp: int = 0  # Stack Pointer

    def run(self, run_current=False, ignore_nop=False):
        try:
            self.ctx = self.current_frame()
            while self.ctx.codes and self.ctx.isp < len(self.ctx.codes):
                codes = self.ctx.codes
                code = self.ctx.codes[self.ctx.isp]
                self.ctx.isp += 1
                # Skip the invalid opcode
                if code == Opcode.INVALID:
                    continue
                arg = None
                if Opcode.has_arg(code):
                    arg = (
                        codes[self.ctx.isp]
                        + (codes[self.ctx.isp + 1] << 8)
                        + (codes[self.ctx.isp + 2] << 16)
                    )
                    self.ctx.isp += 3
                info = self.ctx.codes[self.ctx.isp]
                info = typing.cast(tuple, info)
                self.ctx.isp += 1
                self.ctx.update_info(*info)
                action = VM_OP_ACTIONS.get(code)
                if ignore_nop and (code == Opcode.NOP or code == Opcode.SCHEMA_NOP):
                    continue
                if not action:
                    raise Exception(f"invalid opcode {code}")
                action(self, code, arg)
                if code == Opcode.RETURN_VALUE and run_current:
                    break
        except kcl_error.KCLException as err:
            if not err.lineno:
                filename, line, _ = self.get_info()
                errfile = err.pop_err_info()
                errfile.filename, errfile.line_no = filename, line or None
                err.append_err_info(errfile)
            raise err
        except Exception as err:
            if kclvm.config.debug and kclvm.config.verbose > 2:
                raise
            filename, lineno, _ = self.get_info()
            if isinstance(err, AttributeError):
                kcl_error.report_exception(
                    err_type=kcl_error.ErrType.AttributeError_Runtime_TYPE,
                    file_msgs=[kcl_error.ErrFileMsg(filename=filename, line_no=lineno)],
                    arg_msg=str(err),
                )
            if isinstance(err, RecursionError):
                kcl_error.report_exception(
                    err_type=kcl_error.ErrType.RecursionError_TYPE,
                    file_msgs=[kcl_error.ErrFileMsg(filename=filename, line_no=lineno)],
                    arg_msg=str(err),
                )
            kcl_error.report_exception(
                err_type=kcl_error.ErrType.EvaluationError_TYPE,
                file_msgs=[kcl_error.ErrFileMsg(filename=filename, line_no=lineno)],
                arg_msg=str(err),
            )

        return self.ctx.globals

    def get_info(self, with_column=False) -> typing.Tuple[str, int, int]:
        filename, line, column = self.ctx.get_info()
        return filename, line, column if with_column else None

    def get_filename(self) -> str:
        filename, _, _ = self.ctx.get_info()
        return filename

    def load_const(self, index: int) -> KCLObject:
        cst = self.constants[index]
        self.push(cst)
        return cst

    def load_name(self, index: int) -> KCLObject:
        name = self.names[index]
        self.push(self.ctx.globals[name])
        return self.ctx.globals[name]

    def store_name(self, index: int):
        name = self.names[index]
        self.ctx.globals[name] = kclvm.compiler.check.check_type.check(
            self.stack_top(), *self.get_info()
        )

    def load_local(self, index: int):
        name = self.names[index]
        self.push(self.ctx.locals[name])

    def store_local(self, index: int):
        name = self.names[index]
        self.ctx.locals[name] = self.stack_top()

    def update_local(self, name: str, value: KCLObject):
        self.ctx.locals[name] = value

    def update_global(self, name: str, value: KCLObject):
        self.ctx.globals[name] = kclvm.compiler.check.check_type.check(
            value, *self.get_info()
        )

    def load_builtin(self, index: int):
        built_obj_list = kclvm.compiler.extension.builtin.get_builtin_func_objects()
        self.push(built_obj_list[index])

    def set_instruction_pointer(self, index: int):
        self.ctx.isp = int(index)

    def stack_top(self) -> KCLObject:
        return self.stack[-1]

    def current_frame(self) -> Frame:
        return self.frames[-1]

    def push_frame(self, frame: Frame, names=None, constants=None):
        self.frames.append(frame)
        self.ctx = self.frames[-1]
        self.frame_index += 1

    def push_frame_using_callable(
        self,
        pkgpath: str,
        func: KCLCompiledFunctionObject,
        args: typing.List[KCLObject],
        kwargs: typing.List[KWArg],
        args_len: int = 0,
    ):
        assert isinstance(func, KCLCompiledFunctionObject)

        filename, line, column = self.get_info()

        ctx_globals = self.frames[-1].globals
        if pkgpath in self.state.globals_table:
            ctx_globals = self.state.globals_table[pkgpath]
            self.cur_run_pkg = pkgpath

        self.push_frame(
            Frame(
                name=func.name,
                codes=func.instructions,
                pkgpath=pkgpath,
                locals={},
                globals=ctx_globals,
                free_vars=[],
            ),
            func.names,
            func.constants,
        )
        arg_index = 0
        for arg in args[args_len:] or []:
            self.push(arg)
        for default_arg in func.params:
            if default_arg.value:
                self.update_local(default_arg.name, default_arg.value)
        # args[3:] - schema args
        for arg in args[:args_len]:
            name = func.params[arg_index].name
            self.update_local(name, arg)
            arg_index += 1
        for kwarg in kwargs or []:
            name = kwarg.name.value
            if name not in [p.name for p in func.params]:
                kcl_error.report_exception(
                    err_type=kcl_error.ErrType.EvaluationError_TYPE,
                    file_msgs=[
                        kcl_error.ErrFileMsg(
                            filename=filename, line_no=line, col_no=column
                        )
                    ],
                    arg_msg=f"schema arguments got an unexpected keyword argument '{name}'",
                )
            self.update_local(name, kwarg.value)

    def pop_frame(self) -> Frame:
        self.last_popped_frame = self.frames.pop()
        self.ctx = self.frames[-1]
        self.frame_index -= 1
        return self.last_popped_frame

    def push_function(self, index: int):
        func_obj = self.constants[index]
        self.push(func_obj)

    def push(self, obj: KCLObject):
        self.stack.append(obj)

    def pop(self):
        self.last_obj = self.stack.pop()
        return self.last_obj

    def peek(self) -> KCLObject:
        return self.stack[-1]

    def peek_nth(self, index: int) -> KCLObject:
        """View the Nth top item on the stack from index 0"""
        return self.stack[-index]

    def top(self) -> KCLObject:
        """Get the top of the VM stack"""
        return self.stack[-1]

    def set_top(self, obj: KCLObject):
        """Set the top of the VM stack"""
        self.stack[-1] = obj

    def clear(self):
        self.stack.clear()

    def last_popped_obj(self) -> KCLObject:
        return self.last_obj

    def debug_stack(self, idx: int, at: int = 0):
        if at > 0:
            print(f"vm.stack[{-1-idx}]={self.stack[-1-idx]} # at({at})")
        else:
            print(f"vm.stack[{-1-idx}]={self.stack[-1-idx]}")

    def debug_locals(self, _arg: int, at: int = 0):
        if at > 0:
            print(f"vm.ctx.locals={self.ctx.locals} # at({at})")
        else:
            print(f"vm.ctx.locals={self.ctx.locals}")

    def debug_globals(self, _arg: int, at: int = 0):
        if at > 0:
            print(f"vm.ctx.globals={self.ctx.globals} # at({at})")
        else:
            print(f"vm.ctx.globals={self.ctx.globals}")

    def debug_names(self, _arg: int, at: int = 0):
        if at > 0:
            print(f"vm.names={self.names} # at({at})")
        else:
            print(f"vm.names={self.names}")


def Run(app: KCLProgram, *, pkg: str = None) -> KCLResult:
    return VirtualMachine.RunApp(app, pkg=pkg)
