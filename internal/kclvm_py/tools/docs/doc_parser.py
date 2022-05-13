"""
Docstring parser for KCL models.

Basically, we follow the docstring standard of numpy doc, use the related tools as underlying libs
and extend the part which it does not cover, such as default value.

Reference:
+ https://numpydoc.readthedocs.io/
"""

import re
import typing
from numpydoc.docscrape import NumpyDocString

import kclvm.kcl.ast as ast
import kclvm.api.object as objpkg
import kclvm.tools.docs.checker as checker
import kclvm.tools.docs.model_pb2 as model

# Try to match type, optional, and default value.
# Note: there is no standard way to define default value in numpydoc, we define it as below.
TYPE_OPTIONAL_DEFAULT_REGEX = re.compile(
    r"(?P<type>.*?)(,?\s*[Dd]efault(?: is | = |: |s to |)\s*(?P<value>.*?))?(,?\s*(?P<optional>optional|\(optional\)|required|\(required\)).?)?$"
)


class SchemaDocParser:
    """Schema doc parser.

    The schema doc is defined in model.SchemaDoc

    :param schema. The schema ast.
    :param doc. The schema docstring
    """

    def __init__(
        self,
        schema: ast.SchemaStmt,
        schema_type: objpkg.KCLSchemaTypeObject,
        doc=None,
        root: str = None,
        checker=checker.SchemaDocStringChecker,
    ):
        self._schema = schema
        self.root = root
        self.schema_type = schema_type

        if doc is None:
            if schema is None:
                raise ValueError("No schema or documentation string given")

            import inspect

            doc = inspect.cleandoc(schema.doc)
        schema_doc = NumpyDocString(doc)

        # Base Schema relation
        base_schema: model.Type = None
        if self._schema.parent_name:
            assert self.schema_type.base
            base_schema = model.Type(
                type_str=self._schema.parent_name.get_name(),
                type_category=model.Type.TypeCategory.SCHEMA,
                schema_type=model.SchemaType(
                    name=self.schema_type.base.name,
                    relative_path=self.schema_type.base.filename.replace(
                        self.root, ".", 1
                    ),
                ),
            )

        doc = model.SchemaDoc(name=schema.name, base_schema=base_schema)

        # Summary
        summary = ""
        for sum_line in schema_doc["Summary"]:
            summary += sum_line + "\n"
        doc.doc = summary

        # Attributes
        # From docstring: collect the attribute map from the Attributes docstring
        doc_attr_map: typing.Dict[str, model.SchemaAttributeDoc] = {}
        for attr in schema_doc["Attributes"]:
            doc_attr_name = attr[0]
            doc_attr_type = attr[1]
            doc_attr_desc = ""
            doc_attr_default = None
            doc_attr_optional = False

            if attr[2]:
                for desc in attr[2]:
                    doc_attr_desc += desc + "\n"

            # To prevent the doc pattern e.g., `name: type`, and modify it with kcl format tool
            if ":" in doc_attr_name:
                name_type_parts = doc_attr_name.split(":", 1)
                if name_type_parts and len(name_type_parts) >= 2:
                    doc_attr_name = name_type_parts[0].strip()
                    doc_attr_type = name_type_parts[1].strip() + doc_attr_type
            doc_attr_name = doc_attr_name.lstrip("$")
            if doc_attr_type:
                match = TYPE_OPTIONAL_DEFAULT_REGEX.match(doc_attr_type)
                if (
                    match is None
                    or match.group(0) == match.group(2)
                    or match.group(0) == match.group(3)
                ):
                    # if there is no type defined, optional(group2) or default(group3)
                    # may be treated as type(group0)
                    # fixme: hacky, find a better way to check type
                    doc_attr_type = ""
                else:
                    doc_attr_type = match.group("type")
                    # reset on all false cases
                    doc_attr_type = "" if not doc_attr_type else doc_attr_type
                if match is not None:
                    doc_attr_default = match.group("value")
                    optional = match.group("optional")
                    if optional is not None:
                        doc_attr_optional = "optional" in optional
            doc_attr_map[doc_attr_name] = model.SchemaAttributeDoc(
                name=doc_attr_name,
                type=model.Type(type_str=doc_attr_type),
                is_optional=doc_attr_optional,
                default_value=doc_attr_default,
                doc=doc_attr_desc,
            )

        # From source code: collect the attribute map from the schema stmt: get each attribute's type, default value, optional info
        code_attr_map: typing.Dict[str, model.SchemaAttributeDoc] = {}
        for attr_name, attr_obj in self.schema_type.attr_obj_map.items():
            if attr_name == objpkg.SCHEMA_SETTINGS_ATTR_NAME:
                # ignore __settings__ attribute
                # todo: show __settings__ config in schema summary info
                continue
            code_attr_type: model.Type = self.type_doc(attr_obj.attr_type)
            # fixme: hacky. the schema type str should be calculated recursively
            if (
                code_attr_type.type_category == model.Type.TypeCategory.SCHEMA
                and attr_obj.attr_node is not None
                and attr_obj.attr_node.type_node is not None
            ):
                attr_node = typing.cast(ast.SchemaAttr, attr_obj.attr_node)
                code_attr_type.type_str = attr_node.type_node.plain_type_str
            code_attr_optional: bool = attr_obj.is_optional
            # todo: get default value from ast
            code_default_value: str = ""
            code_attr_map[attr_name] = model.SchemaAttributeDoc(
                name=attr_name,
                type=code_attr_type,
                is_optional=code_attr_optional,
                default_value=code_default_value,
                doc="",
            )

        # Merge attributes from docstring and source code
        # According to source code:
        # the attribute list, the type and the optional info of each attribute
        # According to docstring:
        # the default value and the description of each attribute
        for attr_name, code_attr in code_attr_map.items():
            code_attr.doc = (
                doc_attr_map[attr_name].doc if attr_name in doc_attr_map else ""
            )
            code_attr.default_value = (
                doc_attr_map[attr_name].default_value
                if attr_name in doc_attr_map
                else ""
            )
            doc.attributes.append(code_attr)

        # Examples
        examples = ""
        for example in schema_doc["Examples"]:
            examples += example + "\n"
        doc.examples = examples
        self.doc = doc
        # Validate schema attr
        if checker:
            checker(
                model.SchemaDoc(
                    name=schema.name,
                    attributes=[code_attr_map[attr] for attr in code_attr_map],
                ),
                model.SchemaDoc(
                    name=schema.name,
                    attributes=[doc_attr_map[attr] for attr in doc_attr_map],
                ),
            ).check()

    def type_doc(self, tpe: objpkg.KCLBaseTypeObject) -> model.Type:
        def _short_schema_tpe(tpe: objpkg.KCLBaseTypeObject) -> str:
            fullname = tpe.type_str()
            parts = fullname.rsplit(".", 2)
            return f"{parts[1]}.{parts[2]}" if len(parts) > 2 else fullname

        def _get_type_str(tpe: objpkg.KCLBaseTypeObject) -> str:
            if not tpe:
                return ""
            if tpe.type_kind() in type_str_mapping:
                return type_str_mapping[tpe.type_kind()](tpe)
            else:
                return tpe.type_str()

        type_str_mapping = {
            objpkg.KCLTypeKind.StrLitKind: lambda t: f'"{t.value}"',
            objpkg.KCLTypeKind.IntLitKind: lambda t: f"{t.value}",
            objpkg.KCLTypeKind.BoolLitKind: lambda t: f"{t.value}",
            objpkg.KCLTypeKind.FloatLitKind: lambda t: f"{t.value}",
            objpkg.KCLTypeKind.NoneKind: lambda t: "None",
            objpkg.KCLTypeKind.SchemaKind: _short_schema_tpe,
            objpkg.KCLTypeKind.ListKind: lambda t: f"[{_get_type_str(t.item_type)}]",
            objpkg.KCLTypeKind.DictKind: lambda t: f"{{{_get_type_str(t.key_type)}: {_get_type_str(t.value_type)}}}"
            if t.value_type
            else f"{{{_get_type_str(t.key_type)}:}}",
            objpkg.KCLTypeKind.UnionKind: lambda t: " | ".join(
                [_get_type_str(inner_type) for inner_type in t.types]
            ),
        }

        type_mapping = {
            # ant type
            objpkg.KCLTypeKind.AnyKind: lambda t: model.Type(
                type_str=t.type_str(), type_category=model.Type.TypeCategory.ANY
            ),
            # builtin type
            objpkg.KCLTypeKind.StrKind: lambda t: model.Type(
                type_str=t.type_str(),
                type_category=model.Type.TypeCategory.BUILTIN,
                builtin_type=model.Type.BuiltinType.STRING,
            ),
            objpkg.KCLTypeKind.IntKind: lambda t: model.Type(
                type_str=t.type_str(),
                type_category=model.Type.TypeCategory.BUILTIN,
                builtin_type=model.Type.BuiltinType.INT,
            ),
            objpkg.KCLTypeKind.BoolKind: lambda t: model.Type(
                type_str=t.type_str(),
                type_category=model.Type.TypeCategory.BUILTIN,
                builtin_type=model.Type.BuiltinType.BOOL,
            ),
            objpkg.KCLTypeKind.FloatKind: lambda t: model.Type(
                type_str=t.type_str(),
                type_category=model.Type.TypeCategory.BUILTIN,
                builtin_type=model.Type.BuiltinType.FLOAT,
            ),
            # lit type
            objpkg.KCLTypeKind.StrLitKind: lambda t: model.Type(
                type_str=_get_type_str(t),
                type_category=model.Type.TypeCategory.LIT,
                lit_type=model.LitType(string_lit=t.value),
            ),
            objpkg.KCLTypeKind.IntLitKind: lambda t: model.Type(
                type_str=_get_type_str(t),
                type_category=model.Type.TypeCategory.LIT,
                lit_type=model.LitType(int_lit=t.value),
            ),
            objpkg.KCLTypeKind.BoolLitKind: lambda t: model.Type(
                type_str=_get_type_str(t),
                type_category=model.Type.TypeCategory.LIT,
                lit_type=model.LitType(bool_lit=t.value),
            ),
            objpkg.KCLTypeKind.FloatLitKind: lambda t: model.Type(
                type_str=_get_type_str(t),
                type_category=model.Type.TypeCategory.LIT,
                lit_type=model.LitType(float_lit=t.value),
            ),
            # name constant type
            objpkg.KCLTypeKind.NoneKind: lambda t: model.Type(
                type_str=_get_type_str(t),
                type_category=model.Type.TypeCategory.NAMED_CONSTANT,
                named_constant=model.Type.NamedConstant.NONE,
            ),
            # number multiplier type
            objpkg.KCLTypeKind.NumberMultiplierKind: lambda t: model.Type(
                type_str=_get_type_str(t),
                type_category=model.Type.TypeCategory.NUMBER_MULTIPLIER,
            ),
            # schema type
            objpkg.KCLTypeKind.SchemaKind: lambda t: model.Type(
                type_str=_get_type_str(t),
                type_category=model.Type.TypeCategory.SCHEMA,
                schema_type=model.SchemaType(
                    name=t.name,
                    relative_path=t.filename.replace(self.root, ".", 1),
                ),
            ),
            # list type
            objpkg.KCLTypeKind.ListKind: lambda t: model.Type(
                type_str=_get_type_str(t),
                type_category=model.Type.TypeCategory.LIST,
                list_type=model.ListType(item_type=self.type_doc(t.item_type)),
            ),
            # dict type
            objpkg.KCLTypeKind.DictKind: lambda t: model.Type(
                type_str=_get_type_str(t),
                type_category=model.Type.TypeCategory.DICT,
                dict_type=model.DictType(
                    key_type=self.type_doc(t.key_type),
                    value_type=self.type_doc(t.value_type),
                ),
            ),
            # union type
            objpkg.KCLTypeKind.UnionKind: lambda t: model.Type(
                type_str=_get_type_str(t),
                type_category=model.Type.TypeCategory.UNION,
                union_type=model.UnionType(
                    types=[self.type_doc(inner_type) for inner_type in t.types]
                ),
            ),
        }
        if tpe.type_kind() in type_mapping:
            return type_mapping[tpe.type_kind()](tpe)
        raise TypeError
