# Copyright 2021 The KCL Authors. All rights reserved.

from dataclasses import dataclass, field
from typing import List, Optional
from enum import IntEnum


class Opcode(IntEnum):
    """Opcode class is used in KCL virtual machine.

    Virtual machine operator code can be divided into two categories
    - Stack opcode
    - Semantics opcode
    """

    INVALID = -1  # Invalid opcode

    POP_TOP = 1  # Stack pops 1 operand from the top of the stack
    ROT_TWO = 2  # Stack rotates 2 operands
    ROT_THREE = 3  # Stack rotates 3 operands
    DUP_TOP = 4  # Copy the reference of the top element of the Stack to a copy and put it on the top of the stack
    DUP_TOP_TWO = 5  # Copy the references of the first two elements on the top of the Stack to one copy and put it on the top of the stack.
    COPY_TOP = 6  # Copy the value of the top element of the Stack to a copy and put it on the top of the stack
    NOP = 9  # No operation

    UNARY_POSITIVE = 10  # +a
    UNARY_NEGATIVE = 11  # -a
    UNARY_NOT = 12  # not a
    UNARY_INVERT = 13  # ~a

    MEMBER_SHIP_AS = 16  # a as b

    BINARY_POWER = 20  # a ** b
    BINARY_MULTIPLY = 21  # a * b
    BINARY_MODULO = 22  # a % b
    BINARY_ADD = 23  # a + b
    BINARY_SUBTRACT = 24  # a - b
    BINARY_SUBSCR = 25  # b[a]
    BINARY_FLOOR_DIVIDE = 26  # a // b
    BINARY_TRUE_DIVIDE = 27  # a / b
    BINARY_LSHIFT = 28  # `a << b`
    BINARY_RSHIFT = 29  # `a >> b`
    BINARY_AND = 30  # `a & b`
    BINARY_XOR = 31  # `a ^ b`
    BINARY_OR = 32  # `a | b`
    BINARY_LOGIC_AND = 33  # `a and b`
    BINARY_LOGIC_OR = 34  # `a or b`

    INPLACE_FLOOR_DIVIDE = 40  # a //= b
    INPLACE_TRUE_DIVIDE = 41  # a /= b
    INPLACE_ADD = 42  # a += b
    INPLACE_SUBTRACT = 43  # `a -= b`
    INPLACE_MULTIPLY = 44  # `a *= b`
    INPLACE_MODULO = 45  # a %= b
    INPLACE_POWER = 46  # a **= b
    INPLACE_LSHIFT = 47  # a <<= b
    INPLACE_RSHIFT = 48  # a >>= b
    INPLACE_AND = 49  # a &= b
    INPLACE_XOR = 50  # a ^= b
    INPLACE_OR = 51  # a |= b

    COMPARE_EQUAL_TO = 60  # a == b
    COMPARE_NOT_EQUAL_TO = 61  # a != b
    COMPARE_LESS_THAN = 62  # a < b
    COMPARE_LESS_THAN_OR_EQUAL_TO = 63  # a <= b
    COMPARE_GREATER_THAN = 64  # a > b
    COMPARE_GREATER_THAN_OR_EQUAL_TO = 65  # a >= b
    COMPARE_IS = 66  # a is b
    COMPARE_IS_NOT = 67  # a is not b
    COMPARE_IN = 68  # a in b
    COMPARE_NOT_IN = 69  # a not in b

    STORE_MAP = 70  # Put a dict entry into the dict object
    STORE_SUBSCR = 71  # Put a subscript of collection
    DELETE_SUBSCR = 72  # Delete a subscript of collection
    BUILD_SCHEMA_CONFIG = 73  # Generate an empty schema config object
    STORE_SCHEMA_CONFIG = 74  # Put a schema config entry into the schema config object

    PRINT_EXPR = 80  # Print expression to stdout
    EMIT_EXPR = 81  # Emit a schema expression to the output

    SCHEMA_NOP = 90  # Expressions in the schema interval operation
    RETURN_VALUE = 91  # Return value in the schema
    RETURN_LAST_VALUE = 92  # Return the last value in the lambda expression

    HAVE_ARGUMENT = 99  # Opcodes from here have an argument

    STORE_NAME = 100  # Index in name list
    UNPACK_SEQUENCE = 101  # Number of sequence items
    GET_ITER = 102  # Get a element from a iterator
    FOR_ITER = 103  # Get a iterator from str/list/dict/schema
    STORE_ATTR = 105  # Index in name list
    DELETE_ATTR = 106  # Delete a attribute
    STORE_GLOBAL = 107  # Store a global variable
    DELETE_GLOBAL = 108  # Delete a global variable
    LOAD_GLOBAL = 109  # Index in name list
    LOAD_CONST = 110  # Index in const list
    LOAD_NAME = 111  # Index in name list
    LOAD_LOCAL = 112  # Local variable number
    STORE_LOCAL = 113  # Local variable number
    DELETE_LOCAL = 114  # Local variable number
    LOAD_FREE = 115  # Load from closure cell
    STORE_FREE = 116  # Store into cell
    DELETE_FREE = 117  # Delete closure cell
    BUILD_TUPLE = 118  # Number of tuple items
    BUILD_LIST = 119  # Number of list items
    BUILD_SET = 120  # Number of set items
    BUILD_MAP = 121  # Always zero for now
    LOAD_ATTR = 123  # Index in name list
    LOAD_BUILT_IN = 124  # Index in built-in list
    IMPORT_NAME = 126  # Index in name list
    COMPARE_OP = 127  # Comparison operator

    JUMP_FORWARD = 130  # Number of bytes to skip
    JUMP_IF_FALSE_OR_POP = 131  # Target byte offset from beginning of code
    JUMP_IF_TRUE_OR_POP = 132  # Target byte offset from beginning of code
    JUMP_ABSOLUTE = 133  # Target byte offset from beginning of code
    POP_JUMP_IF_FALSE = 134  # Target byte offset from beginning of code
    POP_JUMP_IF_TRUE = 135  # Target byte offset from beginning of code

    CALL_FUNCTION = 140  # #args + (#kwargs<<8) CALL_FUNCTION_XXX opcodes defined below depend on this definition
    MAKE_FUNCTION = 141  # #defaults + #kwdefaults<<8 + #annotations<<16
    BUILD_SLICE = 142  # Number of items
    MAKE_CLOSURE = 143  # same as MAKE_FUNCTION
    LOAD_CLOSURE = 144  # Load free variable from closure
    RAISE_VARARGS = 145  # Number of raise arguments (1, 2 or 3)
    RAISE_CHECK = 146  # Expressions in the check block

    LIST_APPEND = 150  # Append a item into the list used in the comprehension
    SET_ADD = 151  # Append a item into the set used in the comprehension
    MAP_ADD = 152  # Append a item into the dict used in the comprehension
    DELETE_ITEM = 153  # Delete a item into the dict used in the filter expression

    MAKE_SCHEMA = 160  # Build schema construct function
    BUILD_SCHEMA = 161  # Build a schema instance
    LOAD_BUILD_SCHEMA = 162  # Load schema
    SCHEMA_ATTR = 163  # Declare a schema attribute
    SCHEMA_LOAD_ATTR = 164  # Load attribute in the schema
    SCHEMA_UPDATE_ATTR = 165  # Update attribute in the schema
    MAKE_DECORATOR = 166  # Build a decorator in the schema

    FORMAT_VALUES = 170  # Format value in the string interpolation

    DEBUG_STACK = 180  # Debug stack
    DEBUG_LOCALS = 181  # Debug VM locals
    DEBUG_GLOBALS = 182  # Debug VM globals
    DEBUG_NAMES = 183  # Debug VM names

    @staticmethod
    def has_arg(code: int) -> bool:
        return code > Opcode.HAVE_ARGUMENT


@dataclass
class Pos:
    number: int = 0
    pos: int = 0
    filename: str = None
    lineno: int = None
    colno: int = None

    def get_pos(self) -> int:
        return self.pos

    def get_number(self) -> int:
        return self.number

    def get_lineno(self) -> int:
        return self.lineno

    def set_lineno(self, lineno: int):
        self.lineno = lineno

    def set_pos(self, number: int, pos: int) -> bool:
        self.number = number
        old_pos = self.pos
        self.pos = pos
        return old_pos != pos


@dataclass
class Instruction(Pos):
    op: Opcode = None

    @staticmethod
    def size() -> int:
        return 1

    def output(self) -> List[int]:
        return [self.op]


@dataclass
class Label(Pos):
    @staticmethod
    def size() -> int:
        return 0

    def output(self) -> List[int]:
        return []

    def stack_effect(self):
        return 0


@dataclass
class InstructionWithArg(Instruction):
    arg: int = 0

    def byte(self, arg) -> int:
        return arg & 0xFF

    @staticmethod
    def size() -> int:
        return 4

    def output(self) -> List[int]:
        out = [
            self.byte(self.op),
            self.byte(self.arg),
            self.byte(self.arg >> 8),
            self.byte(self.arg >> 16),
        ]
        return out


@dataclass
class JumpRel(InstructionWithArg):
    dest: Label = field(default_factory=Label)

    def output(self) -> List[int]:
        self.arg = self.dest.get_pos()
        return super().output()


@dataclass
class JumpAbs(InstructionWithArg):
    dest: Label = field(default_factory=Label)

    def output(self) -> List[int]:
        self.arg = self.dest.get_pos()
        return super().output()


@dataclass
class EmittedInstruction:
    opcode: Opcode = Opcode.INVALID
    position: int = 0


@dataclass
class CompilationScope:
    instructions: list
    last_instruction: Optional[EmittedInstruction] = field(
        default_factory=EmittedInstruction
    )
    previous_instruction: Optional[EmittedInstruction] = field(
        default_factory=EmittedInstruction
    )
