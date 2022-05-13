# Copyright 2021 The KCL Authors. All rights reserved.

from typing import cast, List, Union
from io import StringIO

import kclvm.kcl.ast as ast
import kclvm.kcl.types as types
import kclvm.api.object as objpkg
import kclvm.tools.printer as printer
import kclvm.compiler.parser.parser as parser

from .parser import (
    parse_code,
    Proto,
    Message,
    Field,
    Map,
    Enum,
    EnumField,
    Type,
    KeyType,
    OneOf,
    OneOfField,
)
from .types import (
    PROTOBUF_SCALAR_TYPE_TO_KCL_MAPPING,
    KCL_SCALAR_TYPE_TO_PROTOBUF_TYPE_MAPPING,
    KCL_SCALAR_TYPE_TO_PROTOBUF_KEY_TYPE_MAPPING,
)
from .printer import print_node_to_string


CODE_INPUT = "<input>"
PROTO3_SYNTAX = "proto3"


def protobuf_to_kcl(proto_code: str) -> str:
    """Covert a protobuf code string to a KCL code string.

    Parameters
    ----------
    proto_code : str. The protobuf code string.

    Returns
    -------
    kcl_code : str. The KCL code string.
    """
    kcl_module = protobuf_to_kcl_ast(proto_code)
    buf = StringIO()
    printer.PrintAST(kcl_module, buf)
    return buf.getvalue()


def kcl_to_protobuf(kcl_code: str) -> str:
    """Covert a a KCL code string to a protobuf code string.

    Parameters
    ----------
    kcl_code : str. The KCL code string.

    Returns
    -------
    proto_code : str. The protobuf code string.
    """
    ast_program = parser.LoadProgram(CODE_INPUT, k_code_list=[kcl_code])
    types.ResolveProgram(ast_program)
    main_ast = ast_program.pkgs[ast.Program.MAIN_PKGPATH][0]
    proto = Proto(
        syntax=PROTO3_SYNTAX,
        statements=[
            convert_schema_to_message(schema) for schema in main_ast.GetSchemaList()
        ],
    )
    return print_node_to_string(proto)


def protobuf_to_kcl_ast(proto_code: str) -> ast.Module:
    """Covert a protobuf code string to a KCL code string.

    Parameters
    ----------
    proto_code : str. The protobuf code string.

    Returns
    -------
    kcl_ast : ast.Module. The KCL Module AST node.
    """
    proto = parse_code(proto_code)
    kcl_module = ast.Module(line=1, column=1)
    for statement in proto.statements:
        if isinstance(statement, Message):
            message = cast(Message, statement)
            kcl_module.body.extend(convert_message_to_schema_list(message))
        elif isinstance(statement, Enum):
            enum = cast(Enum, statement)
            kcl_module.body.append(convert_enum_to_type_alias(enum))
        # TODO: Import node on multi proto files
    return kcl_module


def convert_proto_type_to_kcl_type(tpe: Union[Type, str]) -> str:
    """Convert a proto type to a KCL type"""
    return (
        PROTOBUF_SCALAR_TYPE_TO_KCL_MAPPING.get(tpe.value)
        if isinstance(tpe, (Type, KeyType))
        else (tpe or "any")
    )


def convert_kcl_type_to_proto_type(
    tpe: types.Type, is_proto_key_type: bool = False
) -> Union[str, KeyType, Type]:
    """Convert a kcl type to a proto type"""
    if isinstance(tpe, objpkg.KCLNamedTypeObject):
        return tpe.name
    elif isinstance(
        tpe,
        (
            objpkg.KCLIntTypeObject,
            objpkg.KCLFloatTypeObject,
            objpkg.KCLStringTypeObject,
            objpkg.KCLBoolTypeObject,
        ),
    ):
        return (
            KCL_SCALAR_TYPE_TO_PROTOBUF_KEY_TYPE_MAPPING.get(tpe.type_str())
            if is_proto_key_type
            else KCL_SCALAR_TYPE_TO_PROTOBUF_TYPE_MAPPING.get(tpe.type_str())
        )
    raise ValueError(f"Unsupported KCL type {tpe} to proto type")


def convert_message_to_schema_list(message: Message) -> List[ast.Stmt]:
    """Convert proto Message to KCL schema list because a
    proto Message can be nested definition.
    """
    if not message or not isinstance(message, Message):
        raise ValueError(f"Invalid parameter message {message}, expected Message")
    schema = ast.SchemaStmt()
    # Mapping the message name to the schema name
    schema.name = message.name
    results = []
    results.append(schema)
    for node in message.body:
        schema_attr = ast.SchemaAttr()
        if isinstance(node, Field):
            field = cast(Field, node)
            schema_attr.name = field.name
            kcl_type_str = convert_proto_type_to_kcl_type(field.type)
            schema_attr.type_str = (
                f"[{kcl_type_str}]" if field.repeated else kcl_type_str
            )
            schema.body.append(schema_attr)
        elif isinstance(node, Map):
            field = cast(Map, node)
            schema_attr.name = field.name
            kcl_key_type_str = convert_proto_type_to_kcl_type(field.key_type)
            kcl_value_type_str = convert_proto_type_to_kcl_type(field.type)
            schema_attr.type_str = (
                "{" + kcl_key_type_str + ":" + kcl_value_type_str + "}"
            )
            schema.body.append(schema_attr)
        elif isinstance(node, OneOf):
            field = cast(OneOf, node)
            schema_attr.name = field.name
            # OneOfField
            schema_attr.type_str = " | ".join(
                [convert_proto_type_to_kcl_type(field.type) for field in field.fields]
            )
            schema.body.append(schema_attr)
        elif isinstance(node, Message):
            results.extend(convert_message_to_schema_list(node))
    return results


def convert_enum_to_type_alias(enum: Enum) -> ast.TypeAliasStmt:
    """Convert a proto Enum to a KCL TypeAliasStmt"""
    if not enum or not isinstance(enum, Enum):
        raise ValueError(f"Invalid parameter Enum {enum}, expected Enum")
    type_alias = ast.TypeAliasStmt()
    type_alias.type_name = ast.ASTFactory.get_ast_identifier(enum.name)
    values = [str(node.value) for node in enum.body if isinstance(node, EnumField)]
    type_alias.type_value = ast.Type()
    type_alias.type_value.plain_type_str = " | ".join(values)
    return type_alias


def convert_schema_to_message(schema: ast.SchemaStmt) -> Message:
    """Convert KCL schema to a proto Message"""
    if not schema or not isinstance(schema, ast.SchemaStmt):
        raise ValueError(f"Invalid parameter schema {schema}, expected ast.SchemaStmt")
    return Message(
        name=schema.name,
        body=[
            convert_schema_attr_to_message_body(schema_attr, i + 1)
            for i, schema_attr in enumerate(schema.body)
            if isinstance(schema_attr, ast.SchemaAttr)
        ],
    )


def convert_schema_attr_to_message_body(
    schema_attr: ast.SchemaAttr,
    number: int = 1,
) -> Union[Field, Map, OneOf]:
    """Convert KCL schema attr to a proto Message body item"""
    if not schema_attr or not isinstance(schema_attr, ast.SchemaAttr):
        raise ValueError(
            f"Invalid parameter schema {schema_attr}, expected ast.SchemaAttr"
        )
    type_node = types.parse_type_str(schema_attr.type_str)
    if isinstance(type_node, objpkg.KCLDictTypeObject):
        field = Map(
            name=schema_attr.name,
            key_type=convert_kcl_type_to_proto_type(type_node.key_type, True),
            type=convert_kcl_type_to_proto_type(type_node.value_type),
            number=number,
            options=[],
        )
    elif isinstance(type_node, objpkg.KCLUnionTypeObject):
        # Convert union type to one of
        field = OneOf(
            name=schema_attr.name,
            fields=[],
        )
        filed_name_prefix = schema_attr.name
        for i, tpe in enumerate(type_node.types):
            field.fields.append(
                OneOfField(
                    name=filed_name_prefix + f"{i + 1}",
                    number=i + 1,
                    type=types.type_to_kcl_type_annotation_str(tpe),
                    options=[],
                )
            )
    else:
        field = Field(
            name=schema_attr.name,
            type="",
            repeated=False,
            options=[],
            number=number,
        )
        if isinstance(type_node, objpkg.KCLListTypeObject):
            field.repeated = True
            field.type = convert_kcl_type_to_proto_type(type_node.item_type)
        else:
            field.type = convert_kcl_type_to_proto_type(type_node)
    return field
