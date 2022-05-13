"""The `compiler` file mainly contains the function `CompileProgram`
which is used to compile the AST obtained by the parser module into
KCL bytecode.

The KCL compiler is mainly based on `ast.TreeWalker` to implement
traversal of all AST nodes, perform semantic checks and generate
corresponding bytecodes, and implement scope checks based on the
symbol table.

The main compilation process is to use `ast.TreeTransformer` to
preprocess the AST, such as eliminating syntactic sugar, checking
import, VFS path mapping, and configuration merging, etc. Then
generate the corresponding KCL bytecode, which mainly includes opcode,
operand, name memory, object memory, etc. The KCL bytecode is input
into the KCL virtual machine for execution and the result is obtained.

:note: When the definition of any AST node is modified or the AST node
is added/deleted, it is necessary to modify the corresponding processing
in the compiler walk_{AST Name} methods.
:copyright: Copyright 2020 The KCL Authors. All rights reserved.
"""

import typing

from dataclasses import dataclass
from typing import Callable, Any, List, Dict, Optional, Union

import kclvm.kcl.ast as ast
import kclvm.kcl.error as kcl_error
import kclvm.api.object as objpkg
import kclvm.api.object.internal as obj_internal
import kclvm.compiler.vfs as vfs
import kclvm.compiler.extension.builtin as builtin
import kclvm.compiler.astutil.fix as fix
import kclvm.vm.code as vm
import kclvm.unification as unification
import kclvm.tools.query as query

from kclvm.api.object.internal import Undefined
from kclvm.kcl.types import ResolveProgram, ProgramScope, ANY_TYPE, parse_type_str
from kclvm.compiler.build.symtable import SymbolTable, SymbolScope
from kclvm.compiler.build.utils import units
from kclvm.internal.util import CheckRules

from kclvm.compiler.build.data import (
    CMP_OP_MAPPING,
    BIN_OP_MAPPING,
    UNARY_OP_MAPPING,
    ARG_OP_MAPPING,
    EXPR_OP_MAPPING,
    SUBSCR_OP_MAPPING,
    SYMBOL_SCOPE_LOAD_OP_MAPPING,
    SYMBOL_SCOPE_STORE_OP_MAPPING,
    CompilerInternalErrorMeta,
    SchemaConfigMeta,
)


_COMPILE_ERROR = kcl_error.ErrType.CompileError_TYPE
_BODY_ATTR = "body"
_EXPRS_ATTR = "exprs"

LAMBDA_FUNC_NAME = "<lambda>"
RESERVED_IDENTIFIERS = [
    "True",
    "False",
    "None",
    "Undefined",
]
LITERAL_EXPRS = (
    ast.NumberLit,
    ast.StringLit,
    ast.NameConstantLit,
    ast.QuantExpr,
    ast.ListExpr,
    ast.ListComp,
    ast.DictComp,
)


@dataclass
class RuntimeCode(objpkg.KCLObject):
    """
    Runtime code is a temporary structure for storing compilation results.
    """

    names: List[str]
    constants: List[objpkg.KCLObject]
    codes: List[int]

    def type(self) -> objpkg.KCLObjectType:
        """
        Get the object type
        """
        return objpkg.KCLObjectType.RUNTIME_CODE

    def type_str(self) -> str:
        """
        Get the object type
        """
        return "runtime_code"


# -----------------------------------------------------------------------------
# _CompilerBase
# -----------------------------------------------------------------------------


class _CompilerBase(ast.TreeWalker):
    """_ComplierBase function"""

    def __init__(self, filename=""):
        super().__init__()

        self.pkgpath: str = ""

        # File information
        self.filename: str = filename
        self.lineno: int = 0
        self.colno: int = 0
        # Compiler parameters
        self.names: List[str] = []
        self.constants: List[objpkg.KCLObject] = builtin.get_builtin_func_objects()
        # Symbol table
        self.symtable: SymbolTable = SymbolTable.new_with_built_in()
        # Compile scope level
        self.scopes: list = [vm.CompilationScope(instructions=[])]
        # In schema expression level
        self._is_in_schema_exprs: List[bool] = [False]
        # In schema statement level
        self._is_in_schema_stmt: List[bool] = [False]
        # In lambda expression level
        self._is_in_lambda_expr: List[bool] = [False]
        # In if statement
        self._is_in_if_stmt: List[bool] = [False]
        # Local vars
        self._local_vars: List[str] = []
        # Schema func body and check cache
        self._schema_build_cache: Dict[str, objpkg.RuntimeCode] = {}
        # Lambda temp var index
        self._lambda_temp_var_index = 0

    # Base walker functions

    def generic_walk(self, t: ast.AST):
        """Called if no explicit walker function exists for a node."""
        if not isinstance(t, ast.AST):
            kcl_error.report_exception(
                err_type=_COMPILE_ERROR,
                file_msgs=[
                    kcl_error.ErrFileMsg(
                        filename=self.filename,
                        line_no=self.lineno,
                        col_no=self.colno,
                    )
                ],
                arg_msg=CompilerInternalErrorMeta.INVALID_KCL_AST_MSG,
            )
        if hasattr(t, _BODY_ATTR):
            for n in t.body:
                self.walk(n)
        elif hasattr(t, _EXPRS_ATTR):
            for n in t.exprs:
                self.walk(n)
        else:
            self.walk(t)

    def update_line_column(self, t: ast.AST):
        self.filename = t.filename or self.filename
        self.lineno = t.get_line() if t.get_line() else self.lineno
        self.colno = t.get_column() if t.get_column() else self.colno

    def expr_or_load_none(self, t: ast.Expr):
        if t:
            self.expr(t)
        else:
            self.load_constant(None)

    def stmt_or_load_none(self, t: ast.Stmt):
        if t:
            self.stmt(t)
        else:
            self.load_constant(None)

    def expr(self, t: ast.Expr):
        if not t:
            return
        self.update_line_column(t)
        self.walk(t)

    def stmt(self, t: ast.Stmt):
        if not t:
            return
        self.update_line_column(t)
        self.walk(t)

    def exprs(self, exprs: List[ast.Expr]):
        if not exprs:
            return
        assert isinstance(exprs, list)
        for expr in exprs:
            self.expr(expr)

    def stmts(self, stmts: List[ast.Stmt]):
        if not stmts:
            return
        assert isinstance(stmts, list)
        for stmt in stmts:
            self.stmt(stmt)

    # Util functions

    def get_node_name(self, t: ast.AST):
        """Get the ast.AST node name"""
        assert isinstance(t, ast.AST)
        return t.type

    def raise_err(self, msg: str = ""):
        """Raise a KCL compile error"""
        msg = msg if msg else CompilerInternalErrorMeta.INVALID_KCL_AST_MSG
        kcl_error.report_exception(
            err_type=_COMPILE_ERROR,
            file_msgs=[
                kcl_error.ErrFileMsg(
                    filename=self.filename,
                    line_no=self.lineno,
                    col_no=self.colno,
                )
            ],
            arg_msg=msg,
        )

    # Emit functions

    def enter_scope(self):
        """
        Enter scope such as internal of function and schema
        """
        scope = vm.CompilationScope(instructions=[])
        self.scopes.append(scope)
        self.symtable = SymbolTable.new(self.symtable, self.symtable.num_definitions)

    def leave_scope(self) -> List[int]:
        """
        Leave scope
        """
        if not self.scopes:
            self.raise_err(CompilerInternalErrorMeta.INVALID_GLOBAL_IMPLICIT_SCOPE)
        instructions = self.current_instruction()
        self.scopes.pop()
        self.symtable.outer.num_definitions = self.symtable.num_definitions
        self.symtable = self.symtable.outer
        return instructions  # Return internal scope instructions

    def current_instruction(self) -> List[int]:
        """Get the current instruction"""
        return self.scopes[-1].instructions if self.scopes else []

    def add_instruction(self, ins: List[int]) -> int:
        """
        Add instructions into the current compile scope
        """
        if not self.scopes:
            self.raise_err(CompilerInternalErrorMeta.INVALID_GLOBAL_IMPLICIT_SCOPE)
        pos = len(self.current_instruction())
        if not ins:
            return pos
        self.scopes[-1].instructions.extend(
            ins + [(self.filename, self.lineno, self.colno)]
        )
        return pos

    def add_constant(self, cst: objpkg.KCLObject) -> int:
        """
        Add a KCLObject constant into the constant list
        """
        self.constants.append(cst)
        return len(self.constants)

    def add_name(self, name: str) -> int:
        """
        Add a identifier string into the name list
        """
        self.names.append(name)
        return len(self.names)

    def change_operand(self, op: int, op_pos: int, operand: int):
        """
        Change the operand in index 'op_pos'
        """
        current_instruction = self.current_instruction()
        if op_pos > len(current_instruction) + vm.InstructionWithArg.size() - 1:
            self.raise_err(CompilerInternalErrorMeta.INVALID_OP_POS.format(op_pos))
        assert op == current_instruction[op_pos]
        inst = vm.InstructionWithArg(op=vm.Opcode(op), lineno=self.lineno, arg=operand)
        current_instruction[
            op_pos : op_pos + vm.InstructionWithArg.size()
        ] = inst.output()

    def operand(self, operand1: int = 0, operand2: int = 0, operand3: int = 0):
        """
        Build a total operand using operands
        """
        assert 0 <= operand1 <= 255 and 0 <= operand2 <= 255 and 0 <= operand3 <= 255
        return operand1 + (operand2 << 8) + (operand3 << 16)

    def emit(self, op: vm.Opcode, operand: Optional[int] = None) -> int:
        """
        Generate byte code and operand

        Parameters
        ---------
        op: operation code
        operand: operand
        """
        ins = (
            [op, (self.filename, self.lineno, self.colno)]
            if operand is None
            else [
                op,
                (operand & 0xFF),
                ((operand >> 8) & 0xFF),
                ((operand >> 16) & 0xFF),
                (self.filename, self.lineno, self.colno),
            ]
        )
        pos = len(self.scopes[-1].instructions)
        self.scopes[-1].instructions.extend(ins)
        return pos

    # Emit function object and call

    def make_func_with_content(
        self,
        content_func: Callable,
        name: str,
        args: ast.Arguments = None,
        cached_name: str = None,
    ):
        if not content_func or not isinstance(content_func, Callable):
            raise Exception(f"invalid function body {content_func}")
        free_symbols = []
        argc = 0
        if args:

            def _check_defaults_legal():
                mark = False
                for j, default in enumerate(reversed(args.defaults)):
                    if default is None:
                        mark = True
                    if default is not None and mark is True:
                        kcl_error.report_exception(
                            err_type=kcl_error.ErrType.IllegalArgumentError_Syntax_TYPE,
                            file_msgs=[
                                kcl_error.ErrFileMsg(
                                    filename=args.filename,
                                    line_no=default.line,
                                    col_no=args.args[len(args.defaults) - j - 1].column,
                                    end_col_no=default.end_column,
                                    arg_msg="A default argument",
                                )
                            ],
                            arg_msg="non-default argument follows default argument",
                        )

            CheckRules.check_list_len_equal(
                [args.args, args.defaults, args.type_annotation_list]
            )
            _check_defaults_legal()
            arg_defaults = len([default for default in args.defaults if default])
            argc = self.operand(len(args.args), arg_defaults, 0)
            for i, _ in enumerate(args.args):
                self.load_constant(args.GetArgName(i))
                self.load_constant(args.GetArgType(i))
                self.load_constant(args.GetArgDefault(i))
        if cached_name and cached_name in self._schema_build_cache:
            num_locals = 0
            count = self.add_constant(self._schema_build_cache[cached_name])
        else:
            self.enter_scope()
            if args:
                for arg in args.args:
                    self.symtable.define(arg.get_name(), scope=SymbolScope.LOCAL)
                    self.add_name(arg.get_name())
                    # self.expr(arg)
            content_func(args)
            free_symbols = self.symtable.free_symbols
            instructions = self.leave_scope()
            runtime_code = RuntimeCode(
                codes=instructions,
                names=self.names,
                constants=self.constants,
            )
            count = self.add_constant(runtime_code)
            if cached_name and cached_name not in self._schema_build_cache:
                self._schema_build_cache[cached_name] = runtime_code
        num_locals = len(free_symbols)
        if num_locals > 0:
            for symbol in free_symbols:
                self.emit(vm.Opcode.LOAD_CLOSURE, symbol.index)
                self.emit(vm.Opcode.BUILD_LIST, num_locals)
        # Load code
        self.emit(vm.Opcode.LOAD_CONST, count - 1)
        # Load function/closure name
        self.load_constant(name)
        self.emit(
            vm.Opcode.MAKE_FUNCTION if num_locals == 0 else vm.Opcode.MAKE_CLOSURE, argc
        )

    def emit_call(
        self,
        args: List[ast.Expr],
        keywords: List[ast.Keyword],
    ):
        self.exprs(args)
        check_table = set()
        for kw in keywords:
            if kw in check_table:
                self.raise_err(CompilerInternalErrorMeta.DUPLICATED_KW.format(kw.arg))
            check_table.add(kw)
            self.load_constant(kw.arg.names[0])
            self.expr(kw.value)
        op = vm.Opcode.CALL_FUNCTION
        self.emit(op, len(args) + (len(keywords) << 8))

    # Jump and label Instructions

    def set_jmp(self, op: vm.Opcode, label: vm.Label) -> int:
        inst = None
        if op in [
            vm.Opcode.JUMP_IF_FALSE_OR_POP,
            vm.Opcode.JUMP_IF_TRUE_OR_POP,
            vm.Opcode.JUMP_ABSOLUTE,
            vm.Opcode.POP_JUMP_IF_FALSE,
            vm.Opcode.POP_JUMP_IF_TRUE,
        ]:
            inst = vm.JumpAbs(
                op=op,
                dest=label,
                filename=self.filename,
                lineno=self.lineno,
                colno=self.colno,
            )
        elif op in [
            vm.Opcode.JUMP_FORWARD,
            vm.Opcode.FOR_ITER,
        ]:
            inst = vm.JumpRel(
                op=op,
                dest=label,
                filename=self.filename,
                lineno=self.lineno,
                colno=self.colno,
            )
        else:
            self.raise_err(CompilerInternalErrorMeta.INVALID_ARGED_OP_CODE.format(op))
        pos = self.add_instruction(inst.output())
        return pos

    def op_jmp(self, op: vm.Opcode, label: vm.Label):
        pos = self.set_jmp(op, label)
        label.number = op
        label.pos = pos

    def op_label(self, label: vm.Label) -> int:
        assert isinstance(label, vm.Label)
        if label.number is not None:
            self.change_operand(
                label.number, label.pos, len(self.current_instruction())
            )
        return self.add_instruction(label.output())

    def set_label(self, label: vm.Label):
        assert isinstance(label, vm.Label)
        label.pos = len(self.current_instruction())
        label.number = None
        return self.add_instruction(label.output())

    # Decorator

    def op_decorator(
        self,
        name: str,
        key: str,
        args: ast.CallExpr,
        target: obj_internal.DecoratorTargetType,
    ):
        if not name:
            kcl_error.report_exception(
                err_type=_COMPILE_ERROR,
                arg_msg="decorator name can't be None",
            )
        decorator = objpkg.KCLDecoratorObject(name=name, target=target, key=key)
        if args:
            self.exprs(args.args)
            check_table = set()
            for kw in args.keywords:
                if kw in check_table:
                    self.raise_err(
                        CompilerInternalErrorMeta.DUPLICATED_KW.format(kw.arg)
                    )
                check_table.add(kw)
                self.load_constant(kw.arg.names[0])
                self.expr(kw.value)
            n = self.operand(len(args.args), len(args.keywords))
            self.load_constant(decorator)
            self.emit(vm.Opcode.MAKE_DECORATOR, n)
        else:
            self.load_constant(decorator)
            self.emit(vm.Opcode.MAKE_DECORATOR, 0)

    # Symbol operations

    def store_symbol(
        self,
        name: str,
        *,
        scope: SymbolScope = None,
        do_check: bool = True,
        init_global_name: bool = False,
    ) -> int:
        symbol, exist = self.symtable.define(name, scope)
        symbol.define_count = 0 if init_global_name else (symbol.define_count + 1)
        if exist and do_check:
            if symbol.define_count > 1:
                # Variable name 'a' must be unique in package context
                kcl_error.report_exception(
                    err_type=kcl_error.ErrType.ImmutableCompileError_TYPE,
                    file_msgs=[
                        kcl_error.ErrFileMsg(
                            filename=self.filename,
                            line_no=self.lineno,
                            col_no=self.colno,
                        )
                    ],
                )
        index = self.add_name(name) - 1
        if symbol.scope == SymbolScope.INTERNAL:
            return index
        op = SYMBOL_SCOPE_STORE_OP_MAPPING.get(symbol.scope)
        if not op:
            self.raise_err(CompilerInternalErrorMeta.INVALID_GLOBAL_IMPLICIT_SCOPE)
        self.emit(op, index)
        return index

    def load_symbol(self, name: str, emit: bool = True):
        """
        Identifier symbol e.g., a, b, and c
        """
        if not name:
            self.raise_err(CompilerInternalErrorMeta.INVALID_NAME)
        symbol = self.symtable.resolve(name)
        if not symbol:
            self.raise_err(CompilerInternalErrorMeta.SYMBOL_NOT_DEFINED.format(name))
        code = SYMBOL_SCOPE_LOAD_OP_MAPPING.get(symbol.scope)
        if not code:
            self.raise_err(
                CompilerInternalErrorMeta.INVALID_SYMBOL_SCOPE.format(symbol.scope)
            )
        if emit:
            self.emit(code, symbol.index)
        return symbol

    def op_name(self, op: vm.Opcode, name: str):
        self.symtable.define(name, SymbolScope.INTERNAL)
        index = self.add_name(name) - 1
        self.emit(op, index)
        # Leave the inner attr scope, delete the variable from the symbol table.
        self.symtable.delete(name, SymbolScope.INTERNAL)

    # Object constant operations

    def load_constant(self, value: Any):
        """
        Runtime Literal constant e.g., 1, 1.1 and None
        """
        obj = objpkg.to_kcl_obj(value)
        count = self.add_constant(obj)
        self.emit(vm.Opcode.LOAD_CONST, count - 1)

    def compile_program(self, prog: ast.Program) -> Optional[objpkg.KCLProgram]:
        p = objpkg.KCLProgram(
            root=prog.root,
            main=prog.main,
        )
        for pkgpath in prog.pkgs:
            # Symbol table
            self.symtable: SymbolTable = SymbolTable.new_with_built_in()
            self.symtable.num_definitions = len(self.names)
            # Compile scope level
            self.scopes: list = [vm.CompilationScope(instructions=[])]
            self.pkg_scope = self.program_scope.scope_map[pkgpath]
            self.compile(pkgpath, prog.pkgs[pkgpath])
            p.pkgs[pkgpath] = objpkg.KCLBytecode(
                names=self.names,
                constants=self.constants,
                instructions=self.current_instruction(),
            )
        return p

    def compile(self, pkgpath: str, m_list: List[ast.Module]) -> Optional[RuntimeCode]:
        assert pkgpath
        assert m_list
        self.pkgpath = pkgpath

        # Define global names
        for m in m_list:
            self.filename = m.filename
            # Global schema and rule names
            schema_rule_names = {n.name for n in m.GetSchemaAndRuleList()}
            for name in m.global_names:
                self.load_constant(Undefined)
                self.store_symbol(name, init_global_name=True)
                if name not in schema_rule_names:
                    self.symtable.delete(name, SymbolScope.GLOBAL)

        # Do import
        for m in m_list:
            self.filename = m.filename
            for stmt in m.body:
                if isinstance(stmt, ast.ImportStmt):
                    self.update_line_column(stmt)
                    import_spec = typing.cast(ast.ImportStmt, stmt)
                    self.load_constant(0)
                    self.load_constant(None)

                    self.emit(
                        vm.Opcode.IMPORT_NAME,
                        self.store_symbol(
                            import_spec.path,
                            scope=SymbolScope.LOCAL,
                            init_global_name=True,
                        ),
                    )
                    self.store_symbol(
                        f"@{import_spec.path}",
                        scope=SymbolScope.GLOBAL,
                        init_global_name=True,
                    )

        # Define schema type
        for m in m_list:
            self.filename = m.filename
            for stmt in m.body:
                if isinstance(stmt, ast.SchemaStmt):
                    self.stmt(stmt)
                elif isinstance(stmt, ast.RuleStmt):
                    self.stmt(stmt)

        # Define schema type twice
        for m in m_list:
            self.filename = m.filename
            for stmt in m.body:
                if isinstance(stmt, ast.SchemaStmt):
                    self.stmt(stmt)
                elif isinstance(stmt, ast.RuleStmt):
                    self.stmt(stmt)

        # Exec stmt
        for m in m_list:
            self.filename = m.filename
            for stmt in m.body:
                self.stmt(stmt)


# -----------------------------------------------------------------------------
# Compiler
# -----------------------------------------------------------------------------


@dataclass
class Compiler(_CompilerBase):
    """The Compiler class used to build code object, which will be
    consumed by the virtual machine.

    It is mainly composed of code that traverses the tree, and
    bytecode-related functions are defined in _ComplierBase.
    """

    def __init__(self, program_scope: ProgramScope, filename=""):
        super().__init__(filename)
        self.program_scope: ProgramScope = program_scope
        self.pkg_scope = program_scope.scope_map[ast.Program.MAIN_PKGPATH]

    def get_type_from_identifier(self, t: ast.Identifier):
        if not t or not isinstance(t, ast.Identifier):
            return ANY_TYPE
        tpe = parse_type_str(t.get_name())
        if not isinstance(tpe, objpkg.KCLNamedTypeObject):
            return tpe
        if len(t.names) == 1:
            name = t.names[0]
            if name in self.pkg_scope.elems:
                return self.pkg_scope.elems[name].type
            return ANY_TYPE
        elif len(t.names) == 2:
            pkgpath = t.pkgpath
            name = t.names[1]
            if pkgpath in self.pkg_scope.elems:
                tpe = self.pkg_scope.elems[pkgpath].type
                if (
                    not tpe
                    or not isinstance(tpe, objpkg.KCLModuleTypeObject)
                    or tpe.pkgpath not in self.program_scope.scope_map
                    or name not in self.program_scope.scope_map[tpe.pkgpath].elems
                ):
                    return ANY_TYPE
                return self.program_scope.scope_map[tpe.pkgpath].elems[name].type
            return ANY_TYPE
        self.raise_err(msg="Invalid as keyword right identifier")

    # Walker functions
    def walk_Module(self, t: ast.Module):
        assert isinstance(t, ast.Module)
        self.filename = t.filename
        self.stmts(t.body)

    def walk_ExprStmt(self, t: ast.ExprStmt):
        """ast.AST: ExprStmt"""
        assert isinstance(t, ast.ExprStmt)
        exprs = t.exprs
        for expr in exprs:
            # Ignore the doc string
            if isinstance(expr, ast.StringLit):
                continue
            # Insert nop op before the expr statement
            if self._is_in_schema_stmt[-1] and not self._is_in_if_stmt[-1]:
                self.emit(vm.Opcode.SCHEMA_NOP)
            self.expr(expr)
            # Lambda expression temp variable
            if self._is_in_lambda_expr[-1]:
                self.store_symbol(f"@{self._lambda_temp_var_index}")
                self._lambda_temp_var_index += 1
                # Store lambda expr variable and pop the stored value
                self.emit(vm.Opcode.POP_TOP)
            # If it is a literal and pop it from the stack except in the lambda expression
            elif isinstance(expr, LITERAL_EXPRS) or isinstance(expr, ast.CallExpr):
                self.emit(vm.Opcode.POP_TOP)
            elif isinstance(expr, ast.SchemaExpr):
                self.emit(vm.Opcode.EMIT_EXPR)
            # Insert nop op after the expr statement
            if self._is_in_schema_stmt[-1] and not self._is_in_if_stmt[-1]:
                self.emit(vm.Opcode.SCHEMA_NOP)

    def walk_AssertStmt(self, t: ast.AssertStmt):
        """ast.AST: AssertStmt

        Parameters
        ----------
        test: Optional[Expr]
        if_cond: Optional[Expr]
        msg: Optional[Expr]
        """
        assert isinstance(t, ast.AssertStmt) and t.test

        label_if_cond = vm.Label()

        if t.if_cond:
            self.expr(t.if_cond)
            self.op_jmp(vm.Opcode.POP_JUMP_IF_FALSE, label_if_cond)

        self.expr(t.test)
        label = vm.Label()
        self.op_jmp(vm.Opcode.POP_JUMP_IF_TRUE, label)
        self.expr_or_load_none(t.msg)
        self.emit(vm.Opcode.RAISE_VARARGS, 1)
        self.op_label(label)

        if t.if_cond:
            self.op_label(label_if_cond)

    def walk_IfStmt(self, t: ast.IfStmt):
        """ast.AST: IfStmt

        Parameters
        ----------
        - cond: Expr
        - body: List[Stmt]
        - elif_cond: List[Expr]
        - elif_body: List[List[Stmt]]
        - else_body: List[Stmt]

        Instructions:
        ------------
        - vm.Opcode.POP_JUMP_IF_FALSE {body}

        """
        assert isinstance(t, ast.IfStmt)
        assert t.cond
        assert t.body
        self.expr(t.cond)
        self._is_in_if_stmt.append(True)
        jump_if_false_label = vm.Label()
        jump_last_labels = [vm.Label()]
        # If condition
        self.op_jmp(vm.Opcode.POP_JUMP_IF_FALSE, jump_if_false_label)
        self.stmts(t.body)
        self.op_jmp(vm.Opcode.JUMP_FORWARD, jump_last_labels[0])
        self.op_label(jump_if_false_label)
        # Elif list
        for elif_cond, elif_body in zip(t.elif_cond, t.elif_body):
            self.expr(elif_cond)
            jump_elif_false_label = vm.Label()
            self.op_jmp(vm.Opcode.POP_JUMP_IF_FALSE, jump_elif_false_label)
            self.stmts(elif_body)
            jump_last_label = vm.Label()
            self.op_jmp(vm.Opcode.JUMP_FORWARD, jump_last_label)
            jump_last_labels.append(jump_last_label)
            self.op_label(jump_elif_false_label)
        self.stmts(t.else_body)
        # After else
        for label in jump_last_labels:
            self.op_label(label)
        self._is_in_if_stmt.pop()
        if self._is_in_schema_stmt[-1]:
            self.emit(vm.Opcode.SCHEMA_NOP)

    def walk_ImportStmt(self, t: ast.ImportStmt):
        """ast.AST: ImportStmt

        Parameters
        ---------
        - path: str
        - name: str
        - asname: str

        Instructions
        ------
        - vm.Opcode.IMPORT_NAME {symbol_index} 0 0

        StackLayout
        -----------
        TOS
        - asname
        - name
        """
        assert isinstance(t, ast.ImportStmt)
        assert t.pkg_name

        import_spec = typing.cast(ast.ImportStmt, t)
        self.load_constant(0)
        self.load_constant(None)
        if self.pkgpath == "__main__":
            self.emit(
                vm.Opcode.IMPORT_NAME,
                self.store_symbol(
                    import_spec.path,
                    scope=SymbolScope.LOCAL,
                    init_global_name=True,
                ),
            )
            self.store_symbol(
                import_spec.pkg_name,
                scope=SymbolScope.LOCAL,
                init_global_name=True,
            )
        else:
            self.emit(
                vm.Opcode.IMPORT_NAME,
                self.store_symbol(
                    import_spec.path,
                    scope=SymbolScope.GLOBAL,
                    init_global_name=True,
                ),
            )
            self.store_symbol(
                import_spec.pkg_name,
                scope=SymbolScope.GLOBAL,
                init_global_name=True,
            )

    def walk_RuleStmt(self, t: ast.RuleStmt):
        """ast.AST: RuleStmt

        Parameters
        ----------
        - doc: str = ""
        - name: str = ""
        - parent_rules: List[Identifier] = []
        - decorators: List[Decorator] = []
        - checks: List[CheckExpr] = []
        - name_node: Optional[Name] = None
        - args: Optional[Arguments] = None
        - for_host_name: Optional[Identifier] = None

        Stack Layout
        ------------
        TOS
        - 6. index signature
        - 5. decorator list
        - 4. check func
        - 3. schema_body_func
        - 2. mixin type object list
        - 1. parent_type_obj
        - 0. self type object
        BOS
        """
        assert isinstance(t, ast.RuleStmt)
        assert self.pkgpath
        # The schema type object
        schema_type_obj = self.pkg_scope.elems.get(t.name).type.schema_type

        def schema_body_func(args: ast.Arguments):
            # Store magic variables including config, config_meta and schema self pointer
            magic_argument_list = [
                objpkg.SCHEMA_SELF_VALUE_KEY,
                objpkg.SCHEMA_CONFIG_VALUE_KEY,
                objpkg.SCHEMA_CONFIG_META_KEY,
            ]
            for key in magic_argument_list:
                self.store_symbol(key)
                self.emit(vm.Opcode.POP_TOP)
            self.emit(vm.Opcode.SCHEMA_NOP)
            # Pop frame and return the schema object
            self.emit(vm.Opcode.RETURN_VALUE)

        def schema_check_func(_args: ast.Arguments):
            # Store magic variables including config, config_meta and schema self pointer
            magic_argument_list = [
                objpkg.SCHEMA_SELF_VALUE_KEY,
                objpkg.SCHEMA_CONFIG_VALUE_KEY,
                objpkg.SCHEMA_CONFIG_META_KEY,
            ]

            for key in magic_argument_list:
                self.store_symbol(key)
                self.emit(vm.Opcode.POP_TOP)

            for check in t.checks or []:
                self.expr(check)

            self.emit(vm.Opcode.RETURN_VALUE)

        schema_type_obj.attr_obj_map = {}
        schema_type_obj.node_ref = None
        self.load_constant(schema_type_obj)
        # Rule statement has no schema parent name
        self.load_constant(None)
        # Parent rules
        for rule in t.parent_rules or []:
            rule_names = rule.names
            if rule.pkgpath:
                rule_names[0] = f"@{rule.pkgpath}"
            self.load_constant(".".join(rule_names))
        rule_count = len(t.parent_rules) if t.parent_rules else 0
        # In schema level push
        self._is_in_schema_stmt.append(True)
        # Rule statement has not body func
        # Schema body function including schema args, attribute context
        self.make_func_with_content(
            schema_body_func,
            t.name,
            t.args,
            cached_name=schema_type_obj.runtime_type + "body",
        )
        # Rule check expressions
        if t.checks:
            self.make_func_with_content(
                schema_check_func,
                t.name,
                cached_name=schema_type_obj.runtime_type + "check",
            )
        else:
            self.load_constant(None)

        # Decorators
        for decorator in t.decorators or []:
            self.op_decorator(
                decorator.name.get_name(),
                t.name,
                decorator.args,
                obj_internal.DecoratorTargetType.SCHEMA_TYPE,
            )
        decorator_count = len(t.decorators) if t.decorators else 0
        # Rule statement has no index signature
        self.load_constant(None)
        self.emit(
            vm.Opcode.MAKE_SCHEMA,
            self.operand(decorator_count, rule_count, 0),
        )
        # Store the schema type object to the schema name symbol
        self.store_symbol(t.name, init_global_name=True)
        # In schema level pop
        self._is_in_schema_stmt.pop()

    def walk_SchemaStmt(self, t: ast.SchemaStmt):
        """ast.AST: SchemaStmt

        Parameters
        ----------
        - doc: str
        - name: str
        - parent_name: Identifier
        - is_mixin: bool
        - args: Arguments
        - settings: dict
        - mixins: List[str]
        - body: List[Union[SchemaAttr, Stmt]]
        - decorators: List[Decorator]
        - checks: List[CheckExpr]

        Stack Layout
        ------------
        TOS
        - 6. index signature
        - 5. decorator list
        - 4. check func
        - 3. schema_body_func
        - 2. mixin type object list
        - 1. parent_type_obj
        - 0. self type object
        BOS

        vm.Opcode
        ------
        {vm.Opcode.MAKE_SCHEMA} {decorator count} {mixin count} {attr count} -> SchemaTypeObject
        """
        assert isinstance(t, ast.SchemaStmt)
        assert self.pkgpath
        # The schema type object
        schema_type_obj = self.pkg_scope.elems.get(t.name).type.schema_type

        def schema_body_func(args: ast.Arguments):
            # Store magic variables including config, config_meta and schema self pointer
            magic_argument_list = [
                objpkg.SCHEMA_SELF_VALUE_KEY,
                objpkg.SCHEMA_CONFIG_VALUE_KEY,
                objpkg.SCHEMA_CONFIG_META_KEY,
            ]

            for key in magic_argument_list:
                self.store_symbol(key)
                self.emit(vm.Opcode.POP_TOP)

            self.emit(vm.Opcode.SCHEMA_NOP)
            # Emit schema context body including schema attribute declaration and expression
            self.stmts(t.body)
            self.emit(vm.Opcode.SCHEMA_NOP)
            # Pop frame and return the schema object
            self.emit(vm.Opcode.RETURN_VALUE)

        def schema_check_func(_args: ast.Arguments):
            # Store magic variables including config, config_meta and schema self pointer
            magic_argument_list = [
                objpkg.SCHEMA_SELF_VALUE_KEY,
                objpkg.SCHEMA_CONFIG_VALUE_KEY,
                objpkg.SCHEMA_CONFIG_META_KEY,
            ]

            for key in magic_argument_list:
                self.store_symbol(key)
                self.emit(vm.Opcode.POP_TOP)

            for check in t.checks or []:
                self.expr(check)

            self.emit(vm.Opcode.RETURN_VALUE)

        schema_type_obj.attr_obj_map = {}
        schema_type_obj.node_ref = None
        # Get the parent type obj of the schema if exist
        self.load_constant(schema_type_obj)
        self.expr_or_load_none(t.parent_name)
        parent_name_str = t.parent_name.get_name() if t.parent_name else ""
        if parent_name_str.endswith("Mixin"):
            kcl_error.report_exception(
                err_type=kcl_error.ErrType.IllegalInheritError_TYPE,
                file_msgs=[
                    kcl_error.ErrFileMsg(
                        filename=self.filename,
                        line_no=t.line,
                        col_no=t.column,
                        end_col_no=t.end_column,
                    )
                ],
                arg_msg=f"mixin inheritance {parent_name_str} is prohibited",
            )

        # Mixins
        for mixin in t.mixins or []:
            mixin_names = mixin.names
            if mixin.pkgpath:
                mixin_names[0] = f"@{mixin.pkgpath}"

            if not mixin_names[-1].endswith("Mixin"):
                kcl_error.report_exception(
                    err_type=kcl_error.ErrType.MixinNamingError_TYPE,
                    file_msgs=[
                        kcl_error.ErrFileMsg(
                            filename=self.filename,
                            line_no=mixin.line,
                            col_no=mixin.column,
                            end_col_no=mixin.end_column,
                        )
                    ],
                    arg_msg=f"a valid mixin name should end with 'Mixin', got '{mixin_names[-1]}'",
                )

            self.load_constant(".".join(mixin_names))
        mixin_count = len(t.mixins) if t.mixins else 0

        # In schema level push
        self._is_in_schema_stmt.append(True)

        # Schema body function including schema args, attribute context
        self.make_func_with_content(
            schema_body_func,
            t.name,
            t.args,
            cached_name=schema_type_obj.runtime_type + "body",
        )

        # Schema check function
        if t.checks:
            self.make_func_with_content(
                schema_check_func,
                t.name,
                cached_name=schema_type_obj.runtime_type + "check",
            )
        else:
            self.load_constant(None)

        # Decorators
        for decorator in t.decorators or []:
            self.op_decorator(
                decorator.name.get_name(),
                t.name,
                decorator.args,
                obj_internal.DecoratorTargetType.SCHEMA_TYPE,
            )
        decorator_count = len(t.decorators) if t.decorators else 0
        # Index signature
        self.stmt_or_load_none(t.index_signature)
        self.emit(
            vm.Opcode.MAKE_SCHEMA,
            self.operand(decorator_count, mixin_count, 0),
        )
        # Store the schema type object to the schema name symbol
        self.store_symbol(t.name, init_global_name=True)
        # In schema level pop
        self._is_in_schema_stmt.pop()

    def walk_SchemaAttr(self, t: ast.SchemaAttr):
        """ast.AST: SchemaAttr

        Parameters
        ----------
        - doc: str
        - name: str
        - type_str: str
        - is_optional: bool
        - value: Expr
        - decorators: List[Decorator]
        - op: Union[AugOp, Assign]

        StackLayout
        -----------
        TOS
        - decorators
        - types
        - attr_name
        - default
        - is_optional
        - op: vm.Opcode.
        """
        self._local_vars = []
        self.load_constant(ARG_OP_MAPPING.get(t.op))
        # Optional
        self.load_constant(bool(t.is_optional))
        # Final
        self.load_constant(False)
        # Has default
        self.load_constant(bool(t.value))
        # Default value
        self.expr_or_load_none(t.value)
        # Attr name
        self.load_constant(t.name)
        # Attr type
        self.load_constant(t.type_str)
        # Decorators
        for decorator in t.decorators or []:
            self.op_decorator(
                decorator.name.get_name(),
                t.name,
                decorator.args,
                obj_internal.DecoratorTargetType.ATTRIBUTE,
            )
        self.emit(vm.Opcode.SCHEMA_ATTR, len(t.decorators))
        self.emit(vm.Opcode.SCHEMA_NOP)

    def walk_SchemaIndexSignature(self, t: ast.SchemaIndexSignature):
        """ast.AST: SchemaIndexSignature

        Parameters
        ----------
        - key_name: Optional[str] = None
        - key_type: Optional[str] = "str"
        - value_type: Optional[str] = ""
        - value: Optional[Expr] = None
        - any_other: bool = False
        """
        assert isinstance(t, ast.SchemaIndexSignature)
        if not t.key_type or t.key_type not in ["str", "float", "int"]:
            kcl_error.report_exception(
                err_type=kcl_error.ErrType.IndexSignatureError_TYPE,
                file_msgs=[
                    kcl_error.ErrFileMsg(
                        filename=self.filename,
                        line_no=t.get_line(),
                        col_no=t.get_column(),
                        end_col_no=t.get_end_column(),
                    )
                ],
                arg_msg='the index signature parameter type must be "str", "int" or "float"',
            )
        self.expr_or_load_none(t.value)
        self.load_constant(t.any_other)
        self.load_constant(t.key_name)
        self.load_constant(t.value_type)
        self.load_constant(t.key_type)

    def walk_IfExpr(self, t: ast.IfExpr):
        """ast.AST: IfExpr

        Parameters
        ----------
        - cond: Expr
        - body: Expr
        - orelse: Expr
        """
        assert isinstance(t, ast.IfExpr)
        self.expr(t.cond)
        jump_if_false_label = vm.Label()
        self.op_jmp(vm.Opcode.POP_JUMP_IF_FALSE, jump_if_false_label)
        self.expr(t.body)
        jump_last_label = vm.Label()
        self.op_jmp(vm.Opcode.JUMP_FORWARD, jump_last_label)
        self.op_label(jump_if_false_label)
        self.expr(t.orelse)
        self.op_label(jump_last_label)

    def walk_UnaryExpr(self, t: ast.UnaryExpr):
        """ast.AST: UnaryExpr(Expr)

        Parameters
        ----------
        - op: UnaryOp
        - operand: Expr
        """
        assert isinstance(t, ast.UnaryExpr)
        opcode = UNARY_OP_MAPPING.get(t.op)
        if not opcode:
            self.raise_err(CompilerInternalErrorMeta.UNKNOWN_UNARYOP.format(t.op))
        self.expr(t.operand)
        self.emit(opcode)

    def walk_BinaryExpr(self, t: ast.BinaryExpr):
        """ast.AST: BinaryExpr

        Parameters
        ----------
        - left: Expr
        - right: Expr
        - op: BinaryOperator

        StackLayout
        -----------
        TOS
        - right
        - left
        """
        assert isinstance(t, ast.BinaryExpr) and t.left and t.right and t.op
        op = BIN_OP_MAPPING.get(t.op)
        if not op:
            self.raise_err(CompilerInternalErrorMeta.UNKNOWN_BINOP.format(t.op))
        if op == vm.Opcode.BINARY_LOGIC_AND or op == vm.Opcode.BINARY_LOGIC_OR:
            # LogicExpr
            op = (
                vm.Opcode.JUMP_IF_FALSE_OR_POP
                if op == vm.Opcode.BINARY_LOGIC_AND
                else vm.Opcode.JUMP_IF_TRUE_OR_POP
            )
            values = [t.left, t.right]
            label = vm.Label()
            for i, e in enumerate(values):
                self.expr(e)
                if i != len(values) - 1:
                    self.op_jmp(op, label)
            self.op_label(label)
        else:
            # BinaryExpr
            self.expr(t.left)
            if op == vm.Opcode.MEMBER_SHIP_AS:
                type_object = self.get_type_from_identifier(t.right)
                self.load_constant(type_object)
            else:
                self.expr(t.right)
            # Update the op filename/line/column meta
            self.update_line_column(t)
            self.emit(op)

    def walk_SelectorExpr(self, t: ast.SelectorExpr):
        """ast.AST: SelectorExpr

        Parameters
        ----------
        - value: Expr
        - attr: Identifier
        - ctx: ExprContext
        - has_question: bool
        """
        assert isinstance(t, ast.SelectorExpr)
        jump_if_false_label = vm.Label()
        if t.has_question:
            self.expr(t.value)  # value is the condition
            self.op_jmp(vm.Opcode.POP_JUMP_IF_FALSE, jump_if_false_label)

        if t.ctx != ast.ExprContext.AUGSTORE:
            self.expr(t.value)
        op = EXPR_OP_MAPPING.get(t.ctx)
        if not op:
            self.raise_err(
                CompilerInternalErrorMeta.INVALID_PARAM_IN_ATTR.format(t.ctx)
            )
        if t.ctx == ast.ExprContext.AUGLOAD:
            self.emit(vm.Opcode.DUP_TOP)
        elif t.ctx == ast.ExprContext.AUGSTORE:
            self.emit(vm.Opcode.ROT_TWO)

        self.op_name(op, t.attr.get_name())

        if t.has_question:
            jump_last_label = vm.Label()
            self.op_jmp(vm.Opcode.JUMP_FORWARD, jump_last_label)
            self.op_label(jump_if_false_label)
            self.load_constant(None)
            self.op_label(jump_last_label)

    def walk_CallExpr(self, t: ast.CallExpr):
        """ast.AST: CallExpr

        Parameters
        ----------
        - func: Expr
        - args: List[Expr]
        - keywords: List[Keyword]
        """
        assert isinstance(t, ast.CallExpr)
        self.expr(t.func)
        self.emit_call(t.args, t.keywords)

    def walk_Subscript(self, t: ast.Subscript):
        """ast.AST: Subscript

        Parameters
        ----------
        - value: Expr
        - index: Expr
        - lower: Expr
        - upper: Expr
        - step: Expr
        - has_question: bool
        """
        assert isinstance(t, ast.Subscript)
        jump_if_false_label = vm.Label()
        if t.has_question:
            self.expr(t.value)  # value is the condition
            self.op_jmp(vm.Opcode.POP_JUMP_IF_FALSE, jump_if_false_label)

        if t.ctx != ast.ExprContext.AUGSTORE:
            self.expr(t.value)
            if t.index:
                self.expr(t.index)
            else:
                n = 2
                for expr in [t.lower, t.upper]:
                    self.expr_or_load_none(expr)
                if t.step:
                    n += 1
                    self.expr(t.step)
                self.emit(vm.Opcode.BUILD_SLICE, n)
        opcodes = SUBSCR_OP_MAPPING.get(t.ctx)
        if not opcodes:
            self.raise_err(
                CompilerInternalErrorMeta.INVALID_PARAM_IN_SUBSCR.format(t.ctx)
            )
        for op in opcodes:
            self.emit(op)

        if t.has_question:
            jump_last_label = vm.Label()
            self.op_jmp(vm.Opcode.JUMP_FORWARD, jump_last_label)
            self.op_label(jump_if_false_label)
            self.load_constant(None)
            self.op_label(jump_last_label)

    def walk_ParenExpr(self, t: ast.ParenExpr):
        """ast.AST: ParenExpr

        Parameters
        ----------
        - expr: Expr
        """
        assert isinstance(t, ast.ParenExpr)
        self.expr(t.expr)

    def walk_QuantExpr(self, t: ast.QuantExpr):
        """ast.AST: QuantExpr

        Parameters
        ----------
        - target: Expr
        - variables: List[Identifier]
        - op: QuantOperation
        - test: Optional[Expr]
        - if_cond: Optional[Expr]
        - ctx: ExprContext

        Notes
        -----
        For different quantifier operations, results are different
            any/all: bool
            map: list
            filter: list/dict/schema
        """
        assert isinstance(t, ast.QuantExpr)

        # Quantifier expression initial result
        if t.op in [ast.QuantOperation.ALL, ast.QuantOperation.ANY]:
            self.load_constant(t.op == ast.QuantOperation.ALL)
        elif t.op == ast.QuantOperation.MAP:
            self.emit(vm.Opcode.BUILD_LIST, 0)
        elif t.op == ast.QuantOperation.FILTER:
            self.expr(t.target)
        else:
            self.raise_err(CompilerInternalErrorMeta.INVALID_QUANTIFIER_OP.format(t.op))

        # Jump labels
        start = vm.Label()
        end_for = vm.Label()
        all_any_end = vm.Label()

        # Copy collection value to be filtered
        if t.op == ast.QuantOperation.FILTER:
            self.emit(vm.Opcode.COPY_TOP)
            self.emit(vm.Opcode.ROT_TWO)
            self.emit(vm.Opcode.POP_TOP)

        # Iter the loop target
        self.expr(t.target)
        self.emit(vm.Opcode.GET_ITER, len(t.variables))

        # Mark the beginning of for-loop
        self.set_label(start)
        # Declare iter and the mapping end of iter
        self.op_jmp(vm.Opcode.FOR_ITER, end_for)

        # Push loop variables, such as filter k, v in data:'
        key_name = None
        val_name = None
        for i, v in enumerate(t.variables):
            name = v.get_name(False)
            key_name = name if i == 0 else key_name
            val_name = name if i == 1 else val_name
            self.update_line_column(v)
            self.store_symbol(name, scope=SymbolScope.LOCAL)
            self._local_vars.append(name)
            # POP the temp var_key variable
            self.emit(vm.Opcode.POP_TOP)

        # QuantExpr inner or_test [IF or_test]
        label_if_cond = vm.Label()
        # Expression filter jump condition
        if t.if_cond:
            self.expr(t.if_cond)
            self.op_jmp(vm.Opcode.POP_JUMP_IF_FALSE, label_if_cond)

        # Loop body if exist
        if t.test:
            self.expr(t.test)
            if t.op in [ast.QuantOperation.ALL, ast.QuantOperation.ANY]:
                self.op_jmp(
                    vm.Opcode.POP_JUMP_IF_FALSE
                    if t.op == ast.QuantOperation.ALL
                    else vm.Opcode.POP_JUMP_IF_TRUE,
                    all_any_end,
                )
            elif t.op == ast.QuantOperation.MAP:
                # Operand 2 denote the distance of the list to be mapped and TOS
                self.emit(vm.Opcode.LIST_APPEND, 2)
            elif t.op == ast.QuantOperation.FILTER:
                filter_label = vm.Label()
                self.op_jmp(vm.Opcode.POP_JUMP_IF_TRUE, filter_label)
                # Copy the list/dict/schema loop variable
                self.load_symbol(key_name)
                if val_name:
                    self.load_symbol(val_name)
                    self.load_constant(True)
                else:
                    self.load_constant(None)
                    self.load_constant(False)
                # Operand 3 denote the distance of the list to be filtered and TOS
                self.emit(vm.Opcode.DELETE_ITEM, 5)
                self.op_label(filter_label)

        # Expression filter jump label
        if t.if_cond:
            self.op_label(label_if_cond)
        # To next cycle
        self.set_jmp(vm.Opcode.JUMP_ABSOLUTE, start)  # Mark start
        # Mark for-loop else constant
        if t.op in [ast.QuantOperation.ALL, ast.QuantOperation.ANY]:
            self.op_label(all_any_end)
            self.emit(vm.Opcode.POP_TOP)
            # Pop the initial value of the empty all/any value
            self.emit(vm.Opcode.POP_TOP)
            self.load_constant(t.op == ast.QuantOperation.ANY)
        # Mark the end of for-loop
        self.op_label(end_for)

        # Delete temp loop variables
        for v in t.variables:
            name = v.get_name(False)
            self.symtable.delete(name, SymbolScope.LOCAL)
        self._local_vars = []

    def walk_ListExpr(self, t: ast.ListExpr):
        """ast.AST: ListExpr

        Parameters
        ----------
        - elts: List[Expr]
        """
        assert isinstance(t, ast.ListExpr)
        self.exprs(t.elts)
        self.emit(vm.Opcode.BUILD_LIST, len(t.elts))

    def walk_ListIfItemExpr(self, t: ast.ListIfItemExpr):
        """ast.AST: ListIfItemExpr

        Parameters
        ----------
        if_cond: Optional[Expr] = None
        exprs: List[Expr] = []
        orelse: Optional[Expr] = None

        if condition item1 elif item2 else item3
        """
        assert isinstance(t, ast.ListIfItemExpr)
        self.expr(t.if_cond)
        jump_if_false_label = vm.Label()
        jump_last_label = vm.Label()
        self.op_jmp(vm.Opcode.POP_JUMP_IF_FALSE, jump_if_false_label)
        self.exprs(t.exprs)
        self.emit(vm.Opcode.BUILD_LIST, len(t.exprs))
        self.emit(vm.Opcode.UNPACK_SEQUENCE, 1)
        self.op_jmp(vm.Opcode.JUMP_FORWARD, jump_last_label)
        self.op_label(jump_if_false_label)
        if t.orelse:
            # Add the orelse item into the list
            self.expr(t.orelse)
        else:
            # *None denotes do not add None into the list
            self.load_constant(None)
            self.emit(vm.Opcode.UNPACK_SEQUENCE, 1)
        if t.orelse and not isinstance(t.orelse, ast.ListIfItemExpr):
            self.emit(vm.Opcode.UNPACK_SEQUENCE, 1)
        self.op_label(jump_last_label)

    def walk_ConfigExpr(self, t: ast.ConfigExpr):
        """ast.AST: ConfigExpr

        Parameters
        ----------
        - items: List[ConfigEntry]
        """
        assert isinstance(t, ast.ConfigExpr)
        self.op_config_data(t)

    def walk_ConfigIfEntryExpr(self, t: ast.ConfigIfEntryExpr):
        """ast.AST: ConfigIfEntryExpr

        Parameters
        ----------
        if_cond: Optional[Expr] = None
        keys: List[Expr] = []
        values: List[Expr] = []
        operations: List[Expr] = []
        orelse: Optional[Expr]

        if condition: key: value -> **({key: value} if condition else self.expr(orelse))
        """
        assert isinstance(t, ast.ConfigIfEntryExpr)
        self.expr(t.if_cond)
        jump_if_false_label = vm.Label()
        self.op_jmp(vm.Opcode.POP_JUMP_IF_FALSE, jump_if_false_label)
        self.op_config_data_entries(t.keys, t.values, t.operations)
        jump_last_label = vm.Label()
        self.op_jmp(vm.Opcode.JUMP_FORWARD, jump_last_label)
        self.op_label(jump_if_false_label)
        self.expr_or_load_none(t.orelse)
        self.op_label(jump_last_label)

    def walk_StarredExpr(self, t: ast.StarredExpr):
        assert isinstance(t, ast.StarredExpr) and t.value
        self.expr(t.value)
        self.emit(vm.Opcode.UNPACK_SEQUENCE, 1)

    def comp_generator(
        self,
        generators: List[ast.CompClause],
        gen_index: int,
        elt: ast.Expr,
        val: Optional[ast.Expr],
        op: ast.ConfigEntryOperation,
        node: Union[ast.ListComp, ast.DictComp],
    ):
        start = vm.Label()
        end_for = vm.Label()
        gen = generators[gen_index]

        variable_count = len(gen.targets)
        assert 0 < variable_count <= 2

        self.expr(gen.iter)
        self.emit(vm.Opcode.GET_ITER, variable_count)

        self.set_label(start)
        # Declare iter and the mapping end of iter
        self.op_jmp(vm.Opcode.FOR_ITER, end_for)
        # Push target, such as i in 'for i in [1,2]'
        for target in gen.targets:
            target_name = target.get_name(False)
            self.update_line_column(target)
            self.store_symbol(
                target_name, scope=SymbolScope.LOCAL
            )  # Target in for_comp is a local variable
            self._local_vars.append(target_name)
            self.emit(vm.Opcode.POP_TOP)  # POP the temp target variable

        for e in gen.ifs:
            self.expr(e)
            self.set_jmp(vm.Opcode.POP_JUMP_IF_FALSE, start)

        gen_index += 1
        if gen_index >= len(generators):
            if isinstance(node, ast.ListComp):
                self.expr(elt)
                self.emit(vm.Opcode.LIST_APPEND, int(gen_index + 1))
            elif isinstance(node, ast.DictComp):
                self.expr(val)
                self.expr(elt)
                self.load_constant(op)
                self.emit(vm.Opcode.MAP_ADD, int(gen_index + 1))
            else:
                self.raise_err(CompilerInternalErrorMeta.UNKNOWN_COMP.format(node))
        else:
            self.comp_generator(generators, gen_index, elt, val, op, node)
        # To next cycle
        self.set_jmp(vm.Opcode.JUMP_ABSOLUTE, start)  # Mark start
        # Mark the end of for-loop
        self.op_label(end_for)
        for target in gen.targets:
            target_name = target.get_name(False)
            self.symtable.delete(target_name, SymbolScope.LOCAL)
        self._local_vars = []

    def walk_ListComp(self, t: ast.ListComp):
        """ast.AST: ListComp

        Parameters
        ----------
        - elt: Expr
        - generators: List[CompClause]
            - targets: List[Expr]
            - iter: Expr
            - ifs: List[Expr]
        """
        assert isinstance(t, ast.ListComp)
        self.emit(vm.Opcode.BUILD_LIST, 0)
        self.comp_generator(t.generators, 0, t.elt, None, None, t)

    def walk_DictComp(self, t: ast.DictComp):
        """ast.AST: DictComp

        Parameters
        ----------
        - key: Expr
        - value: Expr
        - generators: List[CompClause]
        """
        assert isinstance(t, ast.DictComp)
        self.emit(vm.Opcode.BUILD_MAP, 0)
        self.comp_generator(t.generators, 0, t.key, t.value, t.operation, t)

    def get_schema_conf_meta(
        self,
        n: typing.Optional[ast.Identifier],
        t: ast.ConfigExpr,
    ):
        """Print the schema conf meta"""
        conf_meta = {}
        if n:
            conf_meta[SchemaConfigMeta.FILENAME] = self.filename
            conf_meta[SchemaConfigMeta.LINE] = n.line
            conf_meta[SchemaConfigMeta.COLUMN] = n.column
        if isinstance(t, ast.ConfigExpr):
            for k, v in zip(t.keys, t.values):
                if not k:
                    # Double star unpack expression
                    continue
                if isinstance(k, ast.Identifier):
                    name = k.get_first_name()
                elif isinstance(k, ast.Literal):
                    name = str(k.value)
                else:
                    name = str(k)
                conf_meta[name] = {
                    "lineno": k.get_line(),
                    "columnno": k.get_column(),
                    "filename": k.filename or self.filename,
                    "$conf_meta": self.get_schema_conf_meta(None, v),
                }
        return conf_meta

    def op_config_data_entries(
        self, keys: List[ast.Expr], values: List[ast.Expr], operations: List[int]
    ):
        self.emit(vm.Opcode.BUILD_SCHEMA_CONFIG)
        for key, value, operation in zip(keys, values, operations):
            insert_index_node = None
            is_nest_key = False
            if key is None:
                self.load_constant(None)
                self.expr(value)
                self.emit(vm.Opcode.UNPACK_SEQUENCE, 2)
            else:
                if isinstance(key, ast.Subscript):
                    if isinstance(key.value, ast.Identifier) and isinstance(
                        key.index, ast.NumberLit
                    ):
                        insert_index_node = key.index
                        key = key.value
                if isinstance(key, ast.Identifier):
                    if len(key.names) == 1:
                        name = key.get_name(False)
                        if name in self._local_vars:
                            self.expr(key)
                        else:
                            self.load_constant(name)
                    else:
                        is_nest_key = True
                        self.load_constant(key.get_name())
                else:
                    self.expr(key)
                self.expr(value)
            self.load_constant(is_nest_key)
            self.load_constant(operation)
            self.expr_or_load_none(insert_index_node)
            self.emit(vm.Opcode.STORE_SCHEMA_CONFIG)

    def op_config_data(self, t: ast.ConfigExpr):
        assert isinstance(t, ast.ConfigExpr)
        self.op_config_data_entries(t.keys, t.values, t.operations)

    def walk_SchemaExpr(self, t: ast.SchemaExpr):
        """ast.AST: SchemaExpr

        Parameters
        ----------
        - name: Identifier
        - config: ConfigExpr
        - schema_args: Arguments
        """
        assert isinstance(t, ast.SchemaExpr)
        # Schema Config data features: 1. Omitted quotes; 2. Nest_key
        config_meta = self.get_schema_conf_meta(t.name, t.config)
        self.exprs(t.args)
        check_table = set()
        for kw in t.kwargs:
            if kw in check_table:
                self.raise_err(CompilerInternalErrorMeta.DUPLICATED_KW.format(kw.arg))
            check_table.add(kw)
            self.load_constant(kw.arg.names[0])
            self.expr(kw.value)
        self.load_constant(config_meta)
        self._is_in_schema_exprs.append(True)
        self.expr(t.config)
        self.expr(t.name)
        n = len(t.args) + (len(t.kwargs) << 8)
        self.emit(vm.Opcode.BUILD_SCHEMA, n)
        self._is_in_schema_exprs.pop()

    def walk_CheckExpr(self, t: ast.CheckExpr):
        """ast.AST: CheckExpr

        Parameters
        ----------
        - test: Expr
        - if_cond: Expr
        - msg: Expr
        """
        assert isinstance(t, ast.CheckExpr) and t.test

        label_if_cond = vm.Label()

        if t.if_cond:
            self.expr(t.if_cond)
            self.op_jmp(vm.Opcode.POP_JUMP_IF_FALSE, label_if_cond)

        self.expr(t.test)
        label = vm.Label()
        self.op_jmp(vm.Opcode.POP_JUMP_IF_TRUE, label)
        self.expr_or_load_none(t.msg)
        self.emit(vm.Opcode.RAISE_CHECK, 1)
        self.op_label(label)

        if t.if_cond:
            self.op_label(label_if_cond)

    def walk_LambdaExpr(self, t: ast.LambdaExpr):
        """ast.AST: LambdaExpr

        Parameters
        ----------
        - args: Optional[Arguments]
        - return_type_str: Optional[str]
        - return_type_node: Optional[Type]
        - body: List[Stmt]
        """

        def lambda_body_func(args: ast.Arguments):
            # Emit lambda function body
            self.stmts(t.body)
            # Pop frame and return the schema object
            self.emit(vm.Opcode.RETURN_LAST_VALUE)

        self._is_in_lambda_expr.append(True)
        self.make_func_with_content(
            lambda_body_func,
            LAMBDA_FUNC_NAME,
            t.args,
        )
        self._is_in_lambda_expr.pop()

    def walk_Compare(self, t: ast.Compare):
        assert isinstance(t, ast.Compare)
        if len(t.ops) == 0:
            self.raise_err(CompilerInternalErrorMeta.NO_OPS_OR_CMPS)
        if len(t.ops) != len(t.comparators):
            self.raise_err(CompilerInternalErrorMeta.UNEQUAL_OPS_AND_CMPS)
        self.expr(t.left)
        labels = []
        for i in range(len(t.ops)):
            has_next = i < (len(t.ops) - 1)
            self.expr(t.comparators[i])
            if has_next:
                # Duplicates the reference on top of the stack.
                self.emit(vm.Opcode.DUP_TOP)
                # Lifts second and third stack item one position up, moves top down to position three.
                self.emit(vm.Opcode.ROT_THREE)
            else:
                self.emit(vm.Opcode.ROT_TWO)
            if CMP_OP_MAPPING.get(t.ops[i]):
                # Performs a Boolean operation. The operation name can be found in cmp_op[opname].
                self.emit(vm.Opcode.COMPARE_OP, CMP_OP_MAPPING.get(t.ops[i]))
            else:
                self.raise_err(CompilerInternalErrorMeta.UNKNOWN_CMPOP.format(t.ops[i]))
            if has_next:
                # If TOS is false, sets the bytecode counter to target and leaves TOS on the stack.
                # Otherwise (TOS is true), TOS is popped.
                label = vm.Label()
                labels.append(label)
                self.op_jmp(vm.Opcode.JUMP_IF_FALSE_OR_POP, label)
        if len(t.ops) > 1:
            end_label = vm.Label()
            # Increments bytecode counter by end label
            self.op_jmp(vm.Opcode.JUMP_FORWARD, end_label)
            for label in labels:
                self.op_label(label)
            # Swaps the two top-most stack items.
            self.emit(vm.Opcode.ROT_TWO)
            # Removes the TOS item.
            self.emit(vm.Opcode.POP_TOP)
            self.op_label(end_label)

    def walk_Identifier(self, t: ast.Identifier):
        """ast.AST: Identifier

        Parameters
        ----------
        - names: List[Name]
        """
        assert isinstance(t, ast.Identifier) and t.ctx
        names = t.names
        if t.pkgpath:
            names[0] = f"@{t.pkgpath}"

        if len(names) == 1:
            name = names[0]
            if name in RESERVED_IDENTIFIERS:
                self.raise_err(CompilerInternalErrorMeta.INVALID_NAME)

            if t.ctx in [ast.ExprContext.LOAD, ast.ExprContext.AUGLOAD]:
                # must be right value
                if self._is_in_schema_stmt[-1] and name not in self._local_vars:
                    self.symtable.define(name, SymbolScope.INTERNAL)
                    index = self.add_name(name) - 1
                    self.load_constant(name)
                    self.emit(vm.Opcode.SCHEMA_LOAD_ATTR, index)
                    # Leave the inner attr scope, delete the variable from the symbol table.
                    self.symtable.delete(name, SymbolScope.INTERNAL)
                else:
                    self.load_symbol(name)
            elif t.ctx in [ast.ast.ExprContext.AUGSTORE, ast.ast.ExprContext.STORE]:
                if self._is_in_lambda_expr[-1]:
                    # Store lambda expr variable and pop the stored value
                    self.store_symbol(name)
                    self.emit(vm.Opcode.POP_TOP)
                elif self._is_in_schema_stmt[-1]:
                    self.load_constant(name)
                    self.emit(vm.Opcode.SCHEMA_UPDATE_ATTR, 0)
                    if not self._is_in_if_stmt[-1]:
                        self.emit(vm.Opcode.SCHEMA_NOP)
                else:
                    self.store_symbol(name)
            elif t.ctx in [ast.ast.ExprContext.DEL]:
                pass
            else:
                assert False

        elif len(names) > 1:

            if t.ctx != ast.ExprContext.AUGSTORE:
                self.expr(
                    ast.Identifier(
                        names=[names[0]], line=self.lineno, column=self.colno
                    )
                )
            name_pairs = list(zip(names, names[1:]))

            for i, data in enumerate(name_pairs):
                name, attr = data[0], data[1]

                ctx = t.ctx  # TODO: Fix single name context in AST
                if i == 0 and (
                    ctx == ast.ExprContext.STORE or ctx == ast.ExprContext.AUGSTORE
                ):
                    self.store_symbol(name)
                if (
                    t.ctx == ast.ExprContext.STORE
                    and i != (len(name_pairs) - 1)
                    and len(name_pairs) > 1
                ):
                    ctx = ast.ExprContext.LOAD

                op = EXPR_OP_MAPPING.get(ctx)

                if not op:
                    self.raise_err(
                        CompilerInternalErrorMeta.INVALID_PARAM_IN_ATTR.format(ctx)
                    )
                if ctx == ast.ExprContext.AUGLOAD:
                    self.emit(vm.Opcode.DUP_TOP)
                elif ctx == ast.ExprContext.AUGSTORE:
                    self.emit(vm.Opcode.ROT_TWO)

                self.op_name(op, attr)
        else:
            self.raise_err(CompilerInternalErrorMeta.INVALID_NAME)

    def walk_NumberLit(self, t: ast.AST):
        """ast.AST: NumberLit

        Parameters
        ----------
        - value
        """
        assert isinstance(t, ast.NumberLit)

        if t.binary_suffix:
            value = units.cal_num(t.value, t.binary_suffix)
            self.load_constant(
                objpkg.KCLNumberMultiplierObject(
                    value=value,
                    raw_value=t.value,
                    binary_suffix=t.binary_suffix,
                )
            )
        else:
            self.load_constant(t.value)

    def walk_StringLit(self, t: ast.StringLit):
        """ast.AST: StringLit

        Parameters
        ----------
        - value
        """
        assert isinstance(t, ast.StringLit)
        self.load_constant(t.value)

    def walk_NameConstantLit(self, t: ast.NameConstantLit):
        """ast.AST: NameConstantLit

        Parameters
        ----------
        - value
        """
        assert isinstance(t, ast.NameConstantLit)
        self.load_constant(t.value)

    def walk_JoinedString(self, t: ast.JoinedString):
        """ast.AST: JoinedString
        Parameters
        ----------
        - values: List[Union[Expr, StringLit]]
        TOS
        ---
        - format_spec
        - formatted expr list
        Operand
        -------
        Formatted expr list count
        """
        assert isinstance(t, ast.JoinedString)
        assert t.values
        for value in t.values:
            if isinstance(value, ast.FormattedValue):
                self.expr(value.value)
                self.load_constant(value.format_spec)
                self.emit(vm.Opcode.FORMAT_VALUES, 1)
            elif isinstance(value, ast.StringLit):
                self.expr(value)
            elif isinstance(value, ast.Expr):
                self.expr(value)
                self.load_constant(None)
                self.emit(vm.Opcode.FORMAT_VALUES, 1)
            else:
                self.raise_err(
                    CompilerInternalErrorMeta.INVALID_STRING_INTERPOLATION_ITEM
                )
        for i in range(len(t.values) - 1):
            self.emit(vm.Opcode.BINARY_ADD)

    def walk_TypeAliasStmt(self, t: ast.TypeAliasStmt):
        """ast.AST: TypeAliasStmt

        Parameters
        ----------
        - type_name: Identifier
        - type_value: Type
        """
        # TypeAliasStmt has been replaced in the ResolveProgram function,
        # there is no need to do any processing here
        pass

    def walk_UnificationStmt(self, t: ast.UnificationStmt):
        """ast.AST: UnificationStmt

        Parameters
        ----------
        - target: Identifier
        - value: Expr
        """
        self._local_vars = []
        name = t.target.get_name()
        if self._is_in_schema_stmt[-1]:
            # Assign operator
            self.load_constant(None)
            # Optional
            self.load_constant(False)
            # Final
            self.load_constant(False)
            # Has default
            self.load_constant(bool(t.value))
            # Default value
            t.target.set_ctx(ast.ExprContext.LOAD)
            self.expr(t.target)
            self.expr_or_load_none(t.value)
            self.emit(vm.Opcode.INPLACE_OR)
            # Attr name
            self.load_constant(name)
            # Attr type
            self.load_constant(t.value.name.get_name())
            # 0 denotes the decorator count
            self.emit(vm.Opcode.SCHEMA_ATTR, 0)
            self.emit(vm.Opcode.SCHEMA_NOP)
        else:
            if not self.symtable.resolve(name):
                self.expr(t.value)
                self.expr(t.target)
            else:
                t.target.set_ctx(ast.ExprContext.LOAD)
                self.expr(t.target)
                self.expr(t.value)
                self.emit(vm.Opcode.INPLACE_OR)
                t.target.set_ctx(ast.ExprContext.STORE)
                self.expr(t.target)

    def walk_AssignStmt(self, t: ast.AssignStmt):
        """ast.AST: AssignStmt

        Parameters
        ----------
        - targets: List[Identifier]
        - value: Expr
        """
        self._local_vars = []
        # Infer to schema
        if t.type_annotation and isinstance(
            self.get_type_from_identifier(t.targets[0]), objpkg.KCLSchemaTypeObject
        ):
            # Config meta
            self.load_constant({})
            # Config
            self.expr(t.value)
            # Load the schema type
            names = t.type_annotation.split(".")
            self.load_symbol(names[0])
            for name in names[1:]:
                self.op_name(vm.Opcode.LOAD_ATTR, name)
            # Build schema
            self.emit(vm.Opcode.BUILD_SCHEMA, 0)
        else:
            self.expr(t.value)
        for i, target in enumerate(t.targets):
            self.update_line_column(target)
            self.expr(target)

    def walk_AugAssignStmt(self, t: ast.AugAssignStmt):
        """ast.AST: AugAssignStmt

        Parameters
        ----------
        - target: Identifier
        - value: Expr
        - op: AugOp
        """
        assert isinstance(t, ast.AugAssignStmt) and t.target and t.value and t.op
        t.target.set_ctx(ast.ExprContext.LOAD)
        self.expr(t.target)
        self.expr(t.value)
        opcode = ARG_OP_MAPPING.get(t.op)
        if not opcode:
            self.raise_err(CompilerInternalErrorMeta.UNKNOWN_AUG_BINOP.format(t.op))
        self.emit(opcode)
        if t.target.pkgpath:
            kcl_error.report_exception(
                err_type=kcl_error.ErrType.CompileError_TYPE,
                file_msgs=[
                    kcl_error.ErrFileMsg(
                        filename=self.filename,
                        line_no=t.target.get_line(),
                        col_no=t.target.get_column(),
                        end_col_no=t.target.get_end_column(),
                    )
                ],
                arg_msg="module '{}' can't be assigned".format(
                    t.target.pkgpath.replace("@", "")
                ),
            )
        t.target.set_ctx(ast.ExprContext.STORE)
        self.expr(t.target)


# -----------------------------------------------------------------------------
# CompileProgram/ResolveProgram
# -----------------------------------------------------------------------------


def FixAndResolveProgram(prog: ast.Program) -> ProgramScope:
    """Fix AST program and resolve it."""
    # Preprocess
    for pkgpath in prog.pkgs:
        # Configuration merge with the same name
        if pkgpath == ast.Program.MAIN_PKGPATH:
            prog.pkgs[pkgpath] = unification.MergeASTList(prog.pkgs[pkgpath])

    # Resolve program including the import check and the type check
    return ResolveProgram(prog)


def CompileProgram(
    prog: ast.Program, enable_cache: bool = True
) -> Optional[objpkg.KCLProgram]:
    """Compile function"""
    if not prog or not isinstance(prog, ast.Program):
        return
    modfile = vfs.LoadModFile(prog.root)
    enable_cache = modfile.build.enable_pkg_cache and enable_cache
    kcl_program = vfs.LoadBytecodeCache(prog.root, prog) if enable_cache else None
    # Preprocess
    for pkgpath in prog.pkgs:
        # Configuration merge with the same name
        if pkgpath == ast.Program.MAIN_PKGPATH:
            # Config merge
            prog.pkgs[pkgpath] = unification.MergeASTList(prog.pkgs[pkgpath])
            # Fix identifier pkgpath
            import_names = {}
            for m in prog.pkgs[pkgpath]:
                fix.fix_qualified_identifier(m, import_names=import_names)

    # Apply command line arguments
    query.ApplyOverrides(prog, prog.cmd_overrides)

    if not kcl_program:
        # Resolve program and get the scope
        scope = ResolveProgram(prog)
        # Compile program using the scope
        kcl_program = Compiler(scope).compile_program(prog)
    if enable_cache:
        vfs.SaveBytecodeCache(prog.root, prog, kcl_program)
    return kcl_program


# -----------------------------------------------------------------------------
# END
# -----------------------------------------------------------------------------
