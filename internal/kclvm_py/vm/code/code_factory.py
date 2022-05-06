# Copyright 2021 The KCL Authors. All rights reserved.

from typing import Optional, List, Tuple
from dataclasses import dataclass, field

from .code import Opcode


@dataclass
class OpcodeContent:
    op: Opcode
    arg: Optional[int]
    arg_list: List[int]
    index: int
    meta: Tuple[str, int, int]

    @property
    def begin_index(self):
        return self.index

    @property
    def end_index(self):
        return self.index + len(self.arg_list) + 1


@dataclass
class OpcodeFactory:
    pkgpath: Optional[str]
    values: List[OpcodeContent] = field(default_factory=list)

    @property
    def begin_index(self):
        return self.values[0].begin_index

    @property
    def end_index(self):
        return self.values[-1].end_index

    def pretty_print(self):
        if not self.values:
            return
        for value in self.values:
            print(
                ">>> ",
                value.op,
                "arg:",
                value.arg,
                "index:",
                value.index,
                "line",
                value.meta[1],
            )

    # ---------------------
    # Static methods
    # ---------------------

    @staticmethod
    def build_from_codes(codes: list, pkgpath: str) -> "OpcodeFactory":
        f = OpcodeFactory(pkgpath=pkgpath)
        isp = 0
        while codes and isp < len(codes):
            code = codes[isp]
            opcode_index = isp
            isp += 1
            # Skip the invalid opcode
            if code == Opcode.INVALID:
                continue
            arg = None
            arg_list = []
            if Opcode.has_arg(code):
                arg = codes[isp] + (codes[isp + 1] << 8) + (codes[isp + 2] << 16)
                arg_list = [codes[isp], codes[isp + 1], codes[isp + 2]]
                isp += 3
            info = codes[isp]
            isp += 1
            f.values.append(
                OpcodeContent(
                    op=Opcode(code),
                    arg=arg,
                    arg_list=arg_list,
                    index=opcode_index,
                    meta=info,
                )
            )
        return f

    @staticmethod
    def to_codes(contents: List[OpcodeContent]) -> list:
        return sum(
            [
                [content.op]
                + (
                    ([content.op] * len(content.arg_list or []))
                    if content.op == Opcode.INVALID
                    else (content.arg_list or [])
                )
                + [content.op if content.op == Opcode.INVALID else content.meta]
                for content in contents or []
            ],
            [],
        )


@dataclass
class SchemaBodyOpcodeFactory(OpcodeFactory):
    schema_name: str = None
    begin_code: List[OpcodeContent] = None
    end_code: List[OpcodeContent] = None

    def validate(self) -> bool:
        return (
            self.values
            and self.values[0].op == Opcode.STORE_LOCAL
            and self.values[-1].op == Opcode.RETURN_VALUE
        )

    def update_start_end_code(self) -> "SchemaBodyOpcodeFactory":
        if not self.validate():
            raise ValueError("Invalid schema opcode factory")
        for i, value in enumerate(self.values):
            if value.op == Opcode.SCHEMA_NOP and not self.begin_code:
                self.begin_code = self.values[:i]
            if value.op == Opcode.RETURN_VALUE:
                self.end_code = [value]
        return self

    def split_to_schema_attr_codes(self) -> List["SchemaBodyOpcodeFactory"]:
        if not self.validate():
            raise ValueError("Invalid schema opcode factory")
        results = []
        last_nop_index = 0
        next_nop_index = -1
        if_stack = [False]
        if not self.end_code or not self.begin_code:
            self.update_start_end_code()
        for i, value in enumerate(self.values):
            if value.op == Opcode.POP_JUMP_IF_FALSE:
                if_stack.append(True)
            if value.op == Opcode.JUMP_FORWARD:
                if_stack.pop()
            if value.op == Opcode.SCHEMA_NOP:
                last_nop_index = i
            if SchemaBodyOpcodeFactory.is_schema_line_op(value.op):
                for j, v in enumerate(self.values[i + 1 :]):
                    if v.op == Opcode.SCHEMA_NOP:
                        next_nop_index = i + j + 1
                        break
                values_append = []
                if if_stack[-1]:
                    for v in self.values[last_nop_index : next_nop_index + 1]:
                        if value.index < v.index < self.values[next_nop_index].index:
                            values_append.append(
                                OpcodeContent(
                                    op=Opcode.INVALID,
                                    arg=v.arg,
                                    arg_list=v.arg_list,
                                    index=v.index,
                                    meta=v.meta,
                                )
                            )
                        else:
                            values_append.append(v)
                else:
                    values_append = self.values[last_nop_index : next_nop_index + 1]
                results.append(
                    SchemaBodyOpcodeFactory(
                        pkgpath=self.pkgpath,
                        schema_name=self.schema_name,
                        values=values_append,
                        begin_code=self.begin_code,
                        end_code=self.end_code,
                    )
                )
        return results

    def to_run_code_list(self) -> list:
        assert self.end_code and self.begin_code
        # Single attribute code
        begin_codes = OpcodeFactory.to_codes(self.begin_code)
        end_codes = OpcodeFactory.to_codes(self.end_code)
        codes = OpcodeFactory.to_codes(self.values)
        start_to_code_count = self.values[0].index - self.begin_code[-1].end_index - 1
        code_to_end_count = self.end_code[0].index - self.values[-1].end_index - 1
        return (
            begin_codes
            + [Opcode.INVALID] * start_to_code_count
            + codes
            + [Opcode.INVALID] * code_to_end_count
            + end_codes
        )

    def pretty_print(self):
        if not self.values:
            return
        RED = "\033[31m"
        GREEN = "\033[32m"
        BLUE = "\033[34m"
        BOLD = "\033[1m"
        RESET = "\033[m"
        found_nop = False
        for value in self.values:
            if value.op == Opcode.SCHEMA_NOP:
                found_nop = True
                print(f"{BOLD}{GREEN}", end="")
            elif value.op == Opcode.RETURN_VALUE or not found_nop:
                print(f"{BOLD}{RED}", end="")
            else:
                print(f"{BOLD}{BLUE}", end="")
            print(
                ">>> ",
                value.op,
                "arg:",
                value.arg,
                "index:",
                value.index,
                "line",
                value.meta[1],
            )
        print(f"{RESET}")

    # ---------------------
    # Static methods
    # ---------------------

    @staticmethod
    def is_schema_line_op(op: int):
        return (
            op == Opcode.SCHEMA_ATTR
            or op == Opcode.SCHEMA_UPDATE_ATTR
            or op == Opcode.STORE_ATTR
        )

    @staticmethod
    def build_from_codes(
        codes: list, pkgpath: str, schema_name: str
    ) -> "SchemaBodyOpcodeFactory":
        ctx = OpcodeFactory.build_from_codes(codes, pkgpath)
        return SchemaBodyOpcodeFactory(
            pkgpath=ctx.pkgpath,
            schema_name=schema_name,
            values=ctx.values,
        )
