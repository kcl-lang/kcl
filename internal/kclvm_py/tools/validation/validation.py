# Copyright 2021 The KCL Authors. All rights reserved.

from typing import Optional, Any, List

import kclvm.kcl.error as kcl_error
import kclvm.kcl.ast as ast
import kclvm.compiler.astutil as astutil
import kclvm.compiler.parser as parser
import kclvm.compiler.extension.builtin.system_module.json as json
import kclvm.compiler.extension.builtin.system_module.yaml as yaml
import kclvm.program.eval as eval


class ValidationMeta:
    TEMP_FILE = "validationTempKCLCode.k"


class ValidationDataFormat:
    """
    KCL validation data formats including yaml and json.
    """

    YAML: str = "yaml"
    JSON: str = "json"
    MAPPING = {
        "YAML": yaml.KMANGLED_decode,
        "JSON": json.KMANGLED_decode,
    }


def validate_code(
    data: str,
    code: str,
    schema: Optional[str] = None,
    attribute_name: str = "value",
    format: str = "json",
    filename: str = None,
) -> bool:
    """Validate the data string using the schema code string, when the parameter
    `schema` is omitted, use the first scheam appeared in the code, when the schema
    if not found, raise an schema not found error.

    Returns a bool result denoting whether validating success, raise an error
    when validating failed because of the file not found error, schema not found
    error, syntax error, check error, etc.

    Parameters
    ----------
    data : str
        A JSON or YAML data string.
    code : str
        A KCL code string.
    schema : str
        The schema name required for verification.
    attribute_name : str
        The validation attribute name, default is `value`.
    format: str, default is "json"
        The data format, suppored json, JSON, yaml and YAML.
    filename: str, default is None
        The filename of the KCL code.

    Examples
    --------
    >>> data = '{"key": "value"}'  # A JSON data string
    >>> code = '''
        schema Person:
            key: str

            check:
                "value" in key  # `key` is required and `key` must contain "value"
        '''
    >>> validate_code(data, code)
    True

    """
    check_validation_para(data, code, format)
    # 1. Parse kcl code string to the AST module.
    module = parser.ParseFile(filename=filename or ValidationMeta.TEMP_FILE, code=code)
    schema_list = astutil.filter_stmt(module, ast.SchemaStmt)
    # 2. Deserialize data str and covert it to a KCL AST node.
    decoder = ValidationDataFormat.MAPPING.get(format.upper())
    value = decoder(data)
    schema_name = schema or (schema_list[0].name if schema_list else None)
    node_list = validate_value_to_ast_node_list(value, schema_name, attribute_name)
    # 3. Insert the value AST node into the module and eval
    module.body = node_list + module.body
    eval.EvalAST(module)
    return True


def validate_code_with_attr_data(
    data: str,
    code: str,
    schema: Optional[str] = None,
    format: str = "json",
) -> bool:
    """Validate the data string using the schema code string, when the parameter
    `schema` is omitted, use the first scheam appeared in the code, when the schema
    if not found, raise an schema not found error.

    Returns a bool result denoting whether validating success, raise an error
    when validating failed because of the file not found error, schema not found
    error, syntax error, check error, etc.

    Parameters
    ----------
    data : str
        A JSON or YAML data string including the attribute key
    code : str
        A KCL code string.
    schema : str
        The schema name required for verification.
    format: str, default is "json"
        The data format, suppored json, JSON, yaml and YAML.

    Examples
    --------
    >>> data = '{"attr": {"key": "value"}}'  # A JSON data string including the attribute name
    >>> code = '''
        schema Person:
            key: str

            check:
                "value" in key  # `key` is required and `key` must contain "value"
        '''
    >>> validate_code_with_attr_data(data, code)
    True
    """
    check_validation_para(data, code, format)
    decoder = ValidationDataFormat.MAPPING.get(format.upper())
    value = decoder(data)
    if not value or not isinstance(value, dict) or len(value) != 1:
        raise ValueError(
            f"Invalid parameter data: {data}, expected a dict with only one attribute"
        )
    attribute_name = list(value.keys())[0]
    data = json.KMANGLED_encode(value[attribute_name])
    return validate_code(
        data=data,
        code=code,
        schema=schema,
        attribute_name=attribute_name,
        format=format,
    )


def check_validation_para(data: str, code: str, format: str):
    if data is None or not isinstance(data, str):
        raise ValueError(f"Invalid parameter data: {data}")
    if code is None or not isinstance(code, str):
        raise ValueError(f"Invalid parameter code: {code}")
    if (
        format is None
        or not isinstance(format, str)
        or format.upper() not in ValidationDataFormat.MAPPING
    ):
        raise ValueError(
            f"Invalid parameter format: {format}, expected one of {ValidationDataFormat.MAPPING.keys()}"
        )


def validate_value_to_ast_node_list(
    value: Any, schema_name: str = None, attribute_name: str = "value"
) -> List[ast.AST]:
    """Covert a validation value to a KCL AST node"""

    def build_assign_node(attribute_name: str, node: ast.AST) -> List[ast.AssignStmt]:
        if not attribute_name:
            raise ValueError(f"Invalid parameter attribute_name: {attribute_name}")
        assign_stmt = ast.AssignStmt()
        assign_stmt.value = node
        assign_stmt.targets = [
            ast.Identifier(
                names=[attribute_name],
                ctx=ast.ExprContext.STORE,
            )
        ]
        return [assign_stmt]

    if isinstance(value, (int, float, bool, str, list, tuple, set, dict)):
        node = value_to_ast(value, schema_name)
        return build_assign_node(attribute_name, node)
    else:
        kcl_error.report_exception(
            err_type=kcl_error.ErrType.CompileError_TYPE,
            arg_msg=f"invalid validation data value {value}",
        )


def value_to_ast(value: Any, schema_name: Optional[str] = None) -> ast.AST:
    node = None
    if value is None:
        node = ast.NameConstantLit(value=None)
    elif isinstance(value, (list, tuple, set)):
        node = ast.ListExpr()
        node.elts = [value_to_ast(v, schema_name) for v in value]
    elif isinstance(value, dict):
        config = ast.ConfigExpr()
        if schema_name:
            node = ast.SchemaExpr()
            node.name = ast.Identifier(names=[schema_name])
            node.config = config
        else:
            node = config
        for k, v in value.items():
            config.items.append(
                ast.ConfigEntry(
                    key=value_to_ast(k, schema_name),
                    value=value_to_ast(v),
                )
            )
    elif isinstance(value, (bool, int, float, str)):
        node = astutil.BuildLitNodeFromValue(value)
    return node
