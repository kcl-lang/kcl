# Copyright 2021 The KCL Authors. All rights reserved.

import io

from typing import List, Dict, Set

import kclvm.compiler.parser as parser
import kclvm.api.object as objpkg
import kclvm.kcl.ast as ast
import kclvm.kcl.types as types
import kclvm.internal.gpyrpc.gpyrpc_pb2 as pb2
import kclvm.tools.printer as printer


def get_schema_type_from_code(
    file: str, code: str, schema_name: str = None
) -> List[pb2.KclType]:
    """Get schema types from a kcl file or code.

    Parameters
    ----------
    file: str
        The kcl filename
    code: str
        The kcl code string
    schema_name: str
        The schema name got, when the schema name is empty, all schemas are returned.

    Returns
    -------
    schema_types: List[pb2.KclType]

    KclType:
        string type = 1;                     // schema, dict, list, str, int, float, bool, int(), float() str(), bool()
        repeated KclType union_types = 2 ;   // union types
        string default = 3;                  // default value

        string schema_name = 4;              // schema name
        string schema_doc = 5;               // schema doc
        map<string, KclType> properties = 6; // schema properties
        repeated string required = 7;        // required schema properties, [property_name1, property_name2]

        KclType key = 8;                     // dict key type
        KclType item = 9;                    // dict/list item type

        int32 line = 10;
    """
    result = []
    program = parser.LoadProgram(
        file or "<input>", k_code_list=[code] if code else None
    )
    for name, o in types.ResolveProgram(program).main_scope.elems.items():
        if isinstance(o.type, objpkg.KCLSchemaDefTypeObject):
            if not schema_name or name == schema_name:
                result.append(kcl_type_obj_to_pb_kcl_type(o.type))
    return result


def kcl_type_obj_to_pb_kcl_type(tpe: types.Type) -> pb2.KclType:
    """any, schema, dict, list, str, int, float, bool, int(), float() str(), bool(), union"""
    if isinstance(tpe, objpkg.KCLSchemaDefTypeObject):
        return kcl_type_obj_to_pb_kcl_type(tpe.schema_type)
    elif isinstance(tpe, objpkg.KCLSchemaTypeObject):
        return schema_type_obj_to_pb_kcl_type(tpe)
    elif isinstance(tpe, objpkg.KCLAnyTypeObject):
        return pb2.KclType(
            type="any",
        )
    elif isinstance(tpe, objpkg.KCLDictTypeObject):
        return pb2.KclType(
            type="dict",
            key=kcl_type_obj_to_pb_kcl_type(tpe.key_type),
            item=kcl_type_obj_to_pb_kcl_type(tpe.value_type),
        )
    elif isinstance(tpe, objpkg.KCLListTypeObject):
        return pb2.KclType(
            type="list",
            item=kcl_type_obj_to_pb_kcl_type(tpe.item_type),
        )
    elif isinstance(
        tpe,
        (
            objpkg.KCLIntTypeObject,
            objpkg.KCLFloatTypeObject,
            objpkg.KCLStringTypeObject,
            objpkg.KCLBoolTypeObject,
            objpkg.KCLIntLitTypeObject,
            objpkg.KCLFloatLitTypeObject,
            objpkg.KCLStringLitTypeObject,
            objpkg.KCLBoolLitTypeObject,
        ),
    ):
        return pb2.KclType(
            type=tpe.type_str(),
        )
    elif isinstance(tpe, objpkg.KCLUnionTypeObject):
        return pb2.KclType(
            type="union",
            union_types=[kcl_type_obj_to_pb_kcl_type(t) for t in tpe.types],
        )
    return (
        pb2.KclType(
            type=tpe.type_str(),
        )
        if isinstance(tpe, types.Type)
        else None
    )


def schema_type_obj_to_pb_kcl_type(
    schema_type_obj: objpkg.KCLSchemaTypeObject,
) -> pb2.KclType:
    """Convert the schema type object to the protobuf kcl type."""
    return pb2.KclType(
        type="schema",
        schema_name=schema_type_obj.name,
        schema_doc=schema_type_obj.doc,
        properties=get_schema_type_obj_properties(schema_type_obj),
        decorators=[
            kcl_decorator_to_pb_decorator(decorator)
            for decorator in schema_type_obj.node_ref.decorators or []
        ],
        required=list(sorted(get_schema_type_obj_required_attributes(schema_type_obj))),
    )


def get_schema_type_obj_properties(
    schema_type_obj: objpkg.KCLSchemaTypeObject,
) -> Dict[str, pb2.KclType]:
    """Get schema properties from a schema type object"""
    result_map = {}
    base_result_map = (
        get_schema_type_obj_properties(schema_type_obj.base)
        if schema_type_obj.base
        else {}
    )
    for attr, attr_obj in schema_type_obj.attr_obj_map.items():
        if attr != objpkg.SCHEMA_SETTINGS_ATTR_NAME:
            type_node = attr_obj.attr_type
            result_map[attr] = kcl_type_obj_to_pb_kcl_type(type_node)
            result_map[attr].default = (
                value_to_string(attr_obj.attr_node.value)
                if attr_obj.attr_node.value
                else ""
            )
            result_map[attr].decorators.extend(
                [
                    kcl_decorator_to_pb_decorator(decorator)
                    for decorator in attr_obj.attr_node.decorators
                ]
                if isinstance(attr_obj.attr_node, ast.SchemaAttr)
                else []
            )
    base_result_map.update(result_map)
    line = 1
    for k in base_result_map:
        base_result_map[k].line = line
        line += 1
    return base_result_map


def get_schema_type_obj_required_attributes(
    schema_type_obj: objpkg.KCLSchemaTypeObject,
) -> Set[str]:
    """Get the required attributes from the schema type object"""
    base_attribute_set = (
        get_schema_type_obj_required_attributes(schema_type_obj.base)
        if schema_type_obj.base
        else set()
    )
    required_attribute_set = {
        attr
        for attr, attr_obj in schema_type_obj.attr_obj_map.items()
        if attr != objpkg.SCHEMA_SETTINGS_ATTR_NAME and not attr_obj.is_optional
    }
    required_attribute_set.update(base_attribute_set)
    return required_attribute_set


def kcl_decorator_to_pb_decorator(node: ast.Decorator) -> pb2.Decorator:
    """Convert the decorator node to the protobuf decorator type."""

    return pb2.Decorator(
        name=node.name.get_name(),
        arguments=[value_to_string(arg) for arg in node.args.args or [] if arg]
        if node.args
        else [],
        keywords={
            keyword.arg.get_name(): value_to_string(keyword.value)
            for keyword in node.args.keywords or []
            if keyword
        }
        if node.args
        else {},
    )


def value_to_string(node: ast.AST) -> str:
    """AST value to string"""
    if isinstance(node, ast.StringLit):
        return node.value
    else:
        buffer = io.StringIO()
        printer.PrintAST(node, out=buffer)
        return buffer.getvalue()
