from kclvm.kcl.ast import BinOp, CmpOp, UnaryOp, AugOp, ExprContext

from kclvm.vm.code import Opcode
from kclvm.compiler.build.symtable import SymbolScope


CMP_OP_MAPPING = {
    CmpOp.Eq: Opcode.COMPARE_EQUAL_TO,
    CmpOp.NotEq: Opcode.COMPARE_NOT_EQUAL_TO,
    CmpOp.Lt: Opcode.COMPARE_LESS_THAN,
    CmpOp.LtE: Opcode.COMPARE_LESS_THAN_OR_EQUAL_TO,
    CmpOp.Gt: Opcode.COMPARE_GREATER_THAN,
    CmpOp.GtE: Opcode.COMPARE_GREATER_THAN_OR_EQUAL_TO,
    CmpOp.Is: Opcode.COMPARE_IS,
    CmpOp.IsNot: Opcode.COMPARE_IS_NOT,
    CmpOp.In: Opcode.COMPARE_IN,
    CmpOp.NotIn: Opcode.COMPARE_NOT_IN,
    CmpOp.Not: Opcode.COMPARE_IS_NOT,  # 'not' => 'is not'
}


BIN_OP_MAPPING = {
    **CMP_OP_MAPPING,
    BinOp.Add: Opcode.BINARY_ADD,
    BinOp.Sub: Opcode.BINARY_SUBTRACT,
    BinOp.Mul: Opcode.BINARY_MULTIPLY,
    BinOp.Div: Opcode.BINARY_TRUE_DIVIDE,
    BinOp.Mod: Opcode.BINARY_MODULO,
    BinOp.Pow: Opcode.BINARY_POWER,
    BinOp.LShift: Opcode.BINARY_LSHIFT,
    BinOp.RShift: Opcode.BINARY_RSHIFT,
    BinOp.BitOr: Opcode.BINARY_OR,
    BinOp.BitXor: Opcode.BINARY_XOR,
    BinOp.BitAnd: Opcode.BINARY_AND,
    BinOp.FloorDiv: Opcode.BINARY_FLOOR_DIVIDE,
    BinOp.And: Opcode.BINARY_LOGIC_AND,
    BinOp.Or: Opcode.BINARY_LOGIC_OR,
    BinOp.As: Opcode.MEMBER_SHIP_AS,
}

UNARY_OP_MAPPING = {
    UnaryOp.Invert: Opcode.UNARY_INVERT,
    UnaryOp.Not: Opcode.UNARY_NOT,
    UnaryOp.UAdd: Opcode.UNARY_POSITIVE,
    UnaryOp.USub: Opcode.UNARY_NEGATIVE,
}

ARG_OP_MAPPING = {
    AugOp.Add: Opcode.INPLACE_ADD,
    AugOp.Sub: Opcode.INPLACE_SUBTRACT,
    AugOp.Mul: Opcode.INPLACE_MULTIPLY,
    AugOp.Div: Opcode.INPLACE_TRUE_DIVIDE,
    AugOp.Mod: Opcode.INPLACE_MODULO,
    AugOp.Pow: Opcode.INPLACE_POWER,
    AugOp.LShift: Opcode.INPLACE_LSHIFT,
    AugOp.RShift: Opcode.INPLACE_RSHIFT,
    AugOp.BitOr: Opcode.INPLACE_OR,
    AugOp.BitXor: Opcode.INPLACE_XOR,
    AugOp.BitAnd: Opcode.INPLACE_AND,
    AugOp.FloorDiv: Opcode.INPLACE_FLOOR_DIVIDE,
}

EXPR_OP_MAPPING = {
    ExprContext.AUGLOAD: Opcode.LOAD_ATTR,
    ExprContext.LOAD: Opcode.LOAD_ATTR,
    ExprContext.AUGSTORE: Opcode.STORE_ATTR,
    ExprContext.STORE: Opcode.STORE_ATTR,
    ExprContext.DEL: Opcode.DELETE_ATTR,
}


SUBSCR_OP_MAPPING = {
    ExprContext.AUGLOAD: [Opcode.DUP_TOP_TWO, Opcode.BINARY_SUBSCR],
    ExprContext.LOAD: [Opcode.BINARY_SUBSCR],
    ExprContext.AUGSTORE: [Opcode.ROT_THREE, Opcode.STORE_SUBSCR],
    ExprContext.STORE: [Opcode.STORE_SUBSCR],
    ExprContext.DEL: [Opcode.DELETE_SUBSCR],
}

SYMBOL_SCOPE_LOAD_OP_MAPPING = {
    SymbolScope.BUILT_IN: Opcode.LOAD_BUILT_IN,
    SymbolScope.LOCAL: Opcode.LOAD_LOCAL,
    SymbolScope.GLOBAL: Opcode.LOAD_NAME,
    SymbolScope.FREE: Opcode.LOAD_FREE,
    SymbolScope.INTERNAL: Opcode.LOAD_NAME,
}

SYMBOL_SCOPE_STORE_OP_MAPPING = {
    SymbolScope.GLOBAL: Opcode.STORE_GLOBAL,
    SymbolScope.LOCAL: Opcode.STORE_LOCAL,
    SymbolScope.FREE: Opcode.STORE_FREE,
}


class CompilerInternalErrorMeta:
    COMPILE_ERR = "compile error {}"
    UNKNOWN_MOD = "unknown Module {}"
    UNKNOWN_STMT = "unknown Stmt {}"
    UNKNOWN_EXPR = "unknown Expr {}"
    UNKNOWN_NUM = "unknown num {}"
    UNKNOWN_NAME_CONST = "unknown NameContext const {}"
    UNKNOWN_BINOP = "unknown BinOp {}"
    UNKNOWN_AUG_BINOP = "unknown Aug BinOp {}"
    UNKNOWN_UNARYOP = "unknown UnaryOp {}"
    UNKNOWN_LOOPTYPE = "unknown loop type {}"
    UNKNOWN_CMPOP = "unknown CompareOp {}"
    UNKNOWN_COMP = "unknown comprehension {}"
    INVALID_KCL_AST_MSG = "invalid KCL AST type"
    INVALID_PARAM_IN_ATTR = "invalid param {} in attribute expression"
    INVALID_PARAM_IN_SUBSCR = "invalid param {} in attribute subscript"
    INVALID_SUBSCRIPT_KIND = "invalid subscript kind {}"
    INVALID_TARGET_IN_LIST_TUPLE = (
        "invalid starred assignment target outside a list or tuple"
    )
    INVALID_STARRED_EXPR = "invalid starred expression outside assignment target "
    INVALID_TWO_STARRED_EXPR = "two starred expressions in assignment"
    INVALID_JMP_CALL = "opJmp called with non jump instruction"
    INVALID_NAME = "NameContext can't be None, True or False"
    INVALID_SYMBOL = "symbol can't be None, True or False"
    INVALID_SYMBOL_SCOPE = "invalid symbol scope {}"
    INVALID_NONE_OP = "opcode can't be none"
    INVALID_ARGED_OP_CODE = "opcode {} can't takes an argument"
    INVALID_OP_CODE = "opcode {} can't be emitted without an argument"
    INVALID_EXPR_CONTEXT = "invalid AST AssignStmt ExprContext value {}"
    INVALID_OP_POS = "invalid opcode pos {}"
    INVALID_MISSING_ARG = "missing arg in Arged opcode"
    INVALID_GLOBAL_IMPLICIT_SCOPE = "not expecting scopeGlobalImplicit in set qualname"
    INVALID_CLOSURE_FREE_SCOPE = (
        "invalid closure scope {} for free var {} in symbol table {}"
    )
    INVALID_GENERATORS = "invalid generators"
    INVALID_STRING_INTERPOLATION_ITEM = "invalid string interpolation item"
    INVALID_QUANTIFIER_OP = "invalid quantifier expression operation {}"
    SYMBOL_NOT_DEFINED = "name '{}' is not defined"
    UNEQUAL_OPS_AND_CMPS = "unequal ops and comparators in compare"
    UNEQUAL_DICT_KV_SIZE = "unequal dict keys len {} and values len {}"
    TOO_MANY_EXPRS_IN_STAR_UNPACK = "too many expressions in star-unpacking assignment"
    FAILED_SET_AUG_ASSIGN_CTX = "can't set context in AugAssign"
    NO_OPS_OR_CMPS = "no ops or comparators in compare"
    NO_SYMBOL_TABLE_FOR_AST = "no symbol table found for ast {}"
    DUPLICATED_KW = "duplicated keyword argument"


class SchemaConfigMeta:
    """
    SchemaConfigMeta defines the names of meta information
    """

    LINE = "$lineno"
    COLUMN = "$columnno"
    FILENAME = "$filename"
