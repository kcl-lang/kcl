from typing import List, Dict

import kclvm.kcl.error as kcl_error
import kclvm.tools.docs.model_pb2 as model


class SchemaDocStringChecker:
    """KCL schema docstring verification

    Verify that the schema docstring is consistent with
    the schema definition

    Parameters
    ----------
    schema: model.SchemaDoc
        The schema AST node

    model: model.SchemaDoc
        The schema document model
    """

    ATTR_NOT_FOUND_IN_DOC = (
        "Missing schema attribute description in schema: '{}', attribute: '{}'"
    )
    ATTR_NOT_FOUND_IN_SCHEMA = (
        "Redundant schema attribute description in schema: '{}', attribute: '{}'"
    )
    ATTR_INFO_INCONSISTENT_TYPE = "Inconsistent schema attribute info: schema: '{}', attribute: '{}', type in docstring: '{}', actual type : '{}'"

    ATTR_INFO_INCONSISTENT_OPTIONAL = "Inconsistent schema attribute info: schema: '{}', attribute: '{}', is_optional in docstring: '{}', actual is_optional : '{}'"

    def __init__(self, source_code: model.SchemaDoc, doc_string: model.SchemaDoc):
        self._source_code = source_code
        self._doc_string = doc_string

    def _check_attribute_diff(
        self,
        attr_map: Dict[str, model.SchemaAttributeDoc],
        attr_doc_map: Dict[str, model.SchemaAttributeDoc],
    ):
        """Verify that the schema docstring Attributes section describes exactly the same attributes as the schema body defines:
        1. all the attributes defined in the schema body must have the corresponding description in the docstring Attributes section
        2. all the attributes described in the docstring Attributes section must have the corresponding definition in the schema body
        """

        attr_not_in_doc = set(attr_map) - set(attr_doc_map)
        attr_not_in_schema = set(attr_doc_map) - set(attr_map)
        if attr_not_in_schema:
            kcl_error.report_warning(
                err_type=kcl_error.ErrType.InvalidDocstring_TYPE,
                arg_msg=self.ATTR_NOT_FOUND_IN_SCHEMA.format(
                    self._source_code.name,
                    ",".join(attr_not_in_schema),
                ),
            )
        if attr_not_in_doc:
            kcl_error.report_warning(
                err_type=kcl_error.ErrType.InvalidDocstring_TYPE,
                arg_msg=self.ATTR_NOT_FOUND_IN_DOC.format(
                    self._source_code.name,
                    ",".join(attr_not_in_doc),
                ),
            )

    def _check_attribute_def(
        self,
        attr_map: Dict[str, model.SchemaAttributeDoc],
        attr_doc_map: Dict[str, model.SchemaAttributeDoc],
    ):
        """Verify that each attribute in the schema docstring Attributes section is consistent with the corresponding attribute in schema body.
        Following features of the attribute will be verified:
        1. the attribute's type
        2. the attribute's optional info
        todo: the default value is not checked, since the code representation can be various, it can only be checked semantically.
        """
        common_attr_list = set(attr_map) & set(attr_doc_map)
        for attr in common_attr_list:
            schema_attr = attr_map[attr]
            schema_attr_doc = attr_doc_map[attr]
            if schema_attr.type.type_str.replace(
                " ", ""
            ) != schema_attr_doc.type.type_str.replace(" ", ""):
                kcl_error.report_warning(
                    err_type=kcl_error.ErrType.InvalidDocstring_TYPE,
                    arg_msg=self.ATTR_INFO_INCONSISTENT_TYPE.format(
                        self._source_code.name,
                        attr,
                        schema_attr_doc.type.type_str,
                        schema_attr.type.type_str,
                    ),
                )
            if schema_attr.is_optional != schema_attr_doc.is_optional:
                kcl_error.report_warning(
                    err_type=kcl_error.ErrType.InvalidDocstring_TYPE,
                    arg_msg=self.ATTR_INFO_INCONSISTENT_OPTIONAL.format(
                        self._source_code.name,
                        attr,
                        schema_attr_doc.is_optional,
                        schema_attr.is_optional,
                    ),
                )

    def _check(
        self,
        schema_attr_list: List[model.SchemaAttributeDoc],
        attr_doc_list: List[model.SchemaAttributeDoc],
    ):
        if not schema_attr_list or not attr_doc_list:
            return
        # Grouped by the schema attribute name
        attr_map = {attr.name: attr for attr in schema_attr_list}
        attr_doc_map = {attr.name: attr for attr in attr_doc_list}
        # Check attribute diff between schema docstring and schema AST
        self._check_attribute_diff(attr_map, attr_doc_map)
        # Check attribute definition between schema docstring and schema AST
        self._check_attribute_def(attr_map, attr_doc_map)

    def check(self):
        if (
            self._source_code is None
            or self._doc_string is None
            or not isinstance(self._source_code, model.SchemaDoc)
            or not isinstance(self._doc_string, model.SchemaDoc)
        ):
            return
        schema_attr_list = self._source_code.attributes
        attr_doc_list = self._doc_string.attributes
        self._check(schema_attr_list, attr_doc_list)
