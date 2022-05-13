# -*- coding: utf-8 -*-

# Parser for protocol buffer .proto files
# Thanks to https://github.com/python-parsy/parsy/blob/master/examples/proto3.py
import enum as stdlib_enum
from string import ascii_letters, digits, hexdigits, octdigits

import attr

from parsy import char_from, from_enum, generate, regex, seq, string

# This file follows the spec at
# https://developers.google.com/protocol-buffers/docs/reference/proto3-spec
# very closely.

# However, because we are parsing into useful objects, we do transformations
# along the way e.g. turning into integers, strings etc. and custom objects.
# Some of the lowest level items have been implemented using 'regex' and converting
# the descriptions to regular expressions. Higher level constructs have been
# implemented using other parsy primitives and combinators.

# Notes:

# 1. Whitespace is very badly defined in the 'spec', so we guess what is meant.
# 2. The spec doesn't allow for comments, and neither does this parser.
#    Other places mention that C++ style comments are allowed. To support that,
#    this parser would need to be changed into split lexing/parsing stages
#    (otherwise you hit issues with comments start markers within string literals).
# 3. Other notes inline.


# Our utilities
def optional_string(s):
    return string(s).times(0, 1).concat()


def convert_octal(s):
    return int(s, 8)


def convert_hex(s):
    return int(s, 16)


def exclude_none(data):
    return [i for i in data if i is not None]


convert_decimal = int


def lexeme(p):
    """
    From a parser (or string), make a parser that consumes
    whitespace on either side.
    """
    if isinstance(p, str):
        p = string(p)
    return regex(r"\s*") >> p << regex(r"\s*")


def is_present(p):
    """
    Given a parser or string, make a parser that returns
    True if the parser matches, False otherwise
    """
    return lexeme(p).optional().map(lambda v: False if v is None else True)


# Our data structures
@attr.s
class Import:
    identifier = attr.ib()
    option = attr.ib()


@attr.s
class Package:
    identifier = attr.ib()


@attr.s
class Option:
    name = attr.ib()
    value = attr.ib()


@attr.s
class Field:
    repeated = attr.ib()
    type = attr.ib()
    name = attr.ib()
    number = attr.ib()
    options = attr.ib()


@attr.s
class OneOfField:
    type = attr.ib()
    name = attr.ib()
    number = attr.ib()
    options = attr.ib()


@attr.s
class OneOf:
    name = attr.ib()
    fields = attr.ib()


@attr.s
class Map:
    key_type = attr.ib()
    type = attr.ib()
    name = attr.ib()
    number = attr.ib()
    options = attr.ib()


@attr.s
class Reserved:
    items = attr.ib()


@attr.s
class Range:
    from_ = attr.ib()
    to = attr.ib()


@attr.s
class EnumField:
    name = attr.ib()
    value = attr.ib()
    options = attr.ib()


@attr.s
class Enum:
    name = attr.ib()
    body = attr.ib()


@attr.s
class Message:
    name = attr.ib()
    body = attr.ib()


@attr.s
class Service:
    name = attr.ib()
    body = attr.ib()


@attr.s
class Rpc:
    name = attr.ib()
    request_stream = attr.ib()
    request_message_type = attr.ib()
    response_stream = attr.ib()
    response_message_type = attr.ib()
    options = attr.ib()


@attr.s
class Proto:
    syntax = attr.ib()
    statements = attr.ib()


# Enums:
class ImportOption(stdlib_enum.Enum):
    WEAK = "weak"
    PUBLIC = "public"


class Type(stdlib_enum.Enum):
    DOUBLE = "double"
    FLOAT = "float"
    INT32 = "int32"
    INT64 = "int64"
    UINT32 = "uint32"
    UINT64 = "uint64"
    SINT32 = "sint32"
    SINT64 = "sint64"
    FIXED32 = "fixed32"
    FIXED64 = "fixed64"
    SFIXED32 = "sfixed32"
    SFIXED64 = "sfixed64"
    BOOL = "bool"
    STRING = "string"
    BYTES = "bytes"


class KeyType(stdlib_enum.Enum):
    INT32 = "int32"
    INT64 = "int64"
    UINT32 = "uint32"
    UINT64 = "uint64"
    SINT32 = "sint32"
    SINT64 = "sint64"
    FIXED32 = "fixed32"
    FIXED64 = "fixed64"
    SFIXED32 = "sfixed32"
    SFIXED64 = "sfixed64"
    BOOL = "bool"
    STRING = "string"


# Some extra constants to avoid typing
SEMI = lexeme(";")
EQ = lexeme("=")
LPAREN = lexeme("(")
RPAREN = lexeme(")")
LBRACE = lexeme("{")
RBRACE = lexeme("}")

# -- Beginning of following spec --
# Letters and digits
letter = char_from(ascii_letters)
decimalDigit = char_from(digits)
octalDigit = char_from(octdigits)
hexDigit = char_from(hexdigits)

# Identifiers

# Compared to spec, we add some '_' prefixed items which are not wrapped in `lexeme`,
# on the assumption that spaces in the middle of identifiers are not accepted.
_ident = (letter + (letter | decimalDigit | string("_")).many().concat()).desc("ident")
ident = lexeme(_ident)
fullIdent = lexeme(ident + (string(".") + ident).many().concat()).desc("fullIdent")
_messageName = _ident
messageName = lexeme(ident).desc("messageName")
_enumName = ident
enumName = lexeme(_enumName).desc("enumName")
fieldName = ident.desc("fieldName")
oneofName = ident.desc("oneofName")
mapName = ident.desc("mapName")
serviceName = ident.desc("serviceName")
rpcName = ident.desc("rpcName")
messageType = (
    optional_string(".") + (_ident + string(".")).many().concat() + _messageName
)
enumType = optional_string(".") + (_ident + string(".")).many().concat() + _enumName

# Integer literals
decimalLit = regex("[1-9][0-9]*").desc("decimalLit").map(convert_decimal)
octalLit = regex("0[0-7]*").desc("octalLit").map(convert_octal)
hexLit = regex("0[x|X][0-9a-fA-F]+").desc("octalLit").map(convert_hex)
intLit = decimalLit | octalLit | hexLit


# Floating-point literals
decimals = r"[0-9]+"
exponent = r"[e|E][+|-]?" + decimals
floatLit = (
    regex(
        r"({decimals}\.({decimals})?({exponent})?)|{decimals}{exponent}|\.{decimals}({exponent})?".format(
            decimals=decimals, exponent=exponent
        )
    )
    .desc("floatLit")
    .map(float)
)


# Boolean
boolLit = (string("true").result(True) | string("false").result(False)).desc("boolLit")


# String literals
hexEscape = regex(r"\\[x|X]") >> regex("[0-9a-fA-F]{2}").map(convert_hex).map(chr)
octEscape = regex(r"\\") >> regex("[0-7]{2}").map(convert_octal).map(chr)
charEscape = regex(r"\\") >> (
    string("a").result("\a")
    | string("b").result("\b")
    | string("f").result("\f")
    | string("n").result("\n")
    | string("r").result("\r")
    | string("t").result("\t")
    | string("v").result("\v")
    | string("\\").result("\\")
    | string("'").result("'")
    | string('"').result('"')
)
escapes = hexEscape | octEscape | charEscape
# Correction to spec regarding " and ' inside quoted strings
strLit = (
    string("'") >> (escapes | regex(r"[^\0\n\'\\]")).many().concat() << string("'")
    | string('"') >> (escapes | regex(r"[^\0\n\"\\]")).many().concat() << string('"')
).desc("strLit")
quote = string("'") | string('"')

# EmptyStatement
emptyStatement = string(";").result(None)


# Signed numbers:
def signedNumberChange(s, num):
    """(Extra compared to spec, to cope with need to produce signed numeric values)"""
    return (-1) if s == "-" else (+1)


sign = regex("[-+]?")
signedIntLit = seq(sign, intLit).combine(signedNumberChange)
signedFloatLit = seq(sign, floatLit).combine(signedNumberChange)


# Constant
# put fullIdent at end to disabmiguate from boolLit
constant = signedIntLit | signedFloatLit | strLit | boolLit | fullIdent

# Syntax
syntax = lexeme("syntax") >> EQ >> quote >> string("proto3") << quote + SEMI

# Import Statement
import_option = from_enum(ImportOption)

import_ = seq(
    lexeme("import") >> import_option.optional().tag("option"),
    lexeme(strLit).tag("identifier") << SEMI,
).combine_dict(Import)

# Package
package = seq(lexeme("package") >> fullIdent << SEMI).map(Package)

# Option
optionName = (ident | (LPAREN >> fullIdent << RPAREN)) + (
    string(".") + ident
).many().concat()
option = seq(
    lexeme("option") >> optionName.tag("name"),
    EQ >> constant.tag("value") << SEMI,
).combine_dict(Option)

# Normal field
type_ = lexeme(from_enum(Type) | messageType | enumType)
fieldNumber = lexeme(intLit)

fieldOption = seq(optionName.tag("name"), EQ >> constant.tag("value")).combine_dict(
    Option
)
fieldOptions = fieldOption.sep_by(lexeme(","), min=1)
fieldOptionList = (
    (lexeme("[") >> fieldOptions << lexeme("]"))
    .optional()
    .map(lambda o: [] if o is None else o)
)

field = seq(
    is_present("repeated").tag("repeated"),
    type_.tag("type"),
    fieldName.tag("name") << EQ,
    fieldNumber.tag("number"),
    fieldOptionList.tag("options") << SEMI,
).combine_dict(Field)


# Oneof and oneof field
oneofField = seq(
    type_.tag("type"),
    fieldName.tag("name") << EQ,
    fieldNumber.tag("number"),
    fieldOptionList.tag("options") << SEMI,
).combine_dict(OneOfField)
oneof = seq(
    lexeme("oneof") >> oneofName.tag("name"),
    LBRACE
    >> (oneofField | emptyStatement).many().map(exclude_none).tag("fields")
    << RBRACE,
).combine_dict(OneOf)

# Map field
keyType = lexeme(from_enum(KeyType))
mapField = seq(
    lexeme("map") >> lexeme("<") >> keyType.tag("key_type"),
    lexeme(",") >> type_.tag("type"),
    lexeme(">") >> mapName.tag("name"),
    EQ >> fieldNumber.tag("number"),
    fieldOptionList.tag("options") << SEMI,
).combine_dict(Map)

# Reserved
range_ = seq(
    lexeme(intLit).tag("from_"),
    (lexeme("to") >> (intLit | lexeme("max"))).optional().tag("to"),
).combine_dict(Range)
ranges = range_.sep_by(lexeme(","), min=1)
# The spec for 'reserved' indicates 'fieldName' here, which is never a quoted string.
# But the example has a quoted string. We have changed it to 'strLit'
fieldNames = strLit.sep_by(lexeme(","), min=1)
reserved = seq(lexeme("reserved") >> (ranges | fieldNames) << SEMI).combine(Reserved)

# Enum definition
enumValueOption = seq(optionName.tag("name") << EQ, constant.tag("value")).combine_dict(
    Option
)
enumField = seq(
    ident.tag("name") << EQ,
    lexeme(intLit).tag("value"),
    (lexeme("[") >> enumValueOption.sep_by(lexeme(","), min=1) << lexeme("]"))
    .optional()
    .map(lambda o: [] if o is None else o)
    .tag("options")
    << SEMI,
).combine_dict(EnumField)
enumBody = (
    LBRACE >> (option | enumField | emptyStatement).many().map(exclude_none) << RBRACE
)
enum = seq(lexeme("enum") >> enumName.tag("name"), enumBody.tag("body")).combine_dict(
    Enum
)


# Message definition
@generate
def message():
    yield lexeme("message")
    name = yield messageName
    body = yield messageBody
    return Message(name=name, body=body)


messageBody = (
    LBRACE
    >> (
        field | enum | message | option | oneof | mapField | reserved | emptyStatement
    ).many()
    << RBRACE
)


# Service definition
rpc = seq(
    lexeme("rpc") >> rpcName.tag("name"),
    LPAREN >> (is_present("stream").tag("request_stream")),
    messageType.tag("request_message_type") << RPAREN,
    lexeme("returns") >> LPAREN >> (is_present("stream").tag("response_stream")),
    messageType.tag("response_message_type") << RPAREN,
    ((LBRACE >> (option | emptyStatement).many() << RBRACE) | SEMI.result([]))
    .optional()
    .map(exclude_none)
    .tag("options"),
).combine_dict(Rpc)

service = seq(
    lexeme("service") >> serviceName.tag("name"),
    LBRACE
    >> (option | rpc | emptyStatement).many().map(exclude_none).tag("body")
    << RBRACE,
).combine_dict(Service)


# Proto file
topLevelDef = message | enum | service
proto = seq(
    syntax.tag("syntax"),
    (import_ | package | option | topLevelDef | emptyStatement)
    .many()
    .map(exclude_none)
    .tag("statements"),
).combine_dict(Proto)


def parse_code(code: str) -> Proto:
    """Parse a proto code string

    Parameters
    ----------
    code : str. The proto code string.

    Returns
    -------
    result : Proto. The proto node structure.
    """
    return proto.parse(code)
