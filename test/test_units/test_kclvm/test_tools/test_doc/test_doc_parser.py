import unittest
import pathlib
import typing

import kclvm.compiler.parser.parser as parser
import kclvm.tools.docs.doc_parser as doc_parser
import kclvm.kcl.types.checker as type_checker
import kclvm.api.object as obj_pkg
import kclvm.tools.docs.model_pb2 as model


_DIR_PATH = pathlib.Path(__file__).parent.joinpath("doc_data") / "source_files"


def resolve(kcl_file: str) -> typing.List[model.SchemaDoc]:
    prog = parser.LoadProgram(kcl_file)
    type_checker.ResolveProgramImport(prog)
    checker = type_checker.TypeChecker(prog, type_checker.CheckConfig())
    checker.check_import(prog.MAIN_PKGPATH)
    checker.init_global_types()
    schemas = prog.pkgs[prog.MAIN_PKGPATH][0].GetSchemaList()

    schema_docs: typing.List[model.SchemaDoc] = []
    for schema in schemas:
        schema_obj_type = checker.scope_map[prog.MAIN_PKGPATH].elems[schema.name].type
        assert isinstance(schema_obj_type, obj_pkg.KCLSchemaDefTypeObject)
        schema_docs.append(
            doc_parser.SchemaDocParser(
                schema=schema,
                schema_type=schema_obj_type.schema_type,
                root=prog.root,
            ).doc
        )
    return schema_docs


class KCLDocGenTest(unittest.TestCase):
    def test_simple_case(self) -> None:
        docs = resolve(_DIR_PATH / "simple.k")
        assert len(docs) == 1
        doc = docs[0]
        assert doc.doc.startswith("Person is a simple schema")
        assert doc.attributes[0].name == "name"
        assert doc.attributes[0].type.type_str == "str"
        assert doc.attributes[0].is_optional is False
        assert doc.attributes[0].default_value == '"Default"'
        assert doc.attributes[0].doc.startswith("A Normal attribute named 'name'")
        assert doc.attributes[1].name == "age"
        assert doc.attributes[1].type.type_str == "int"
        assert doc.attributes[1].is_optional is True
        assert doc.attributes[1].default_value == "18"
        assert doc.attributes[1].doc.startswith("A Normal attribute named 'age'")
        assert doc.examples.startswith("person = Person {")

    def test_good_case(self) -> None:
        docs = resolve(_DIR_PATH / "good.k")
        assert len(docs) == 1
        doc = docs[0]
        assert doc.doc.startswith(
            "Server is the common user interface for long-running"
        )
        assert doc.attributes[0].name == "workloadType"
        assert doc.attributes[0].type.type_str == "str"
        assert doc.attributes[0].is_optional is False
        assert doc.attributes[0].default_value == "Deployment"
        assert doc.attributes[0].doc.startswith(
            "Use this attribute to specify which kind of long-running service you want"
        )
        assert doc.attributes[1].name == "name"
        assert doc.attributes[1].type.type_str == "str"
        assert doc.attributes[1].is_optional is False
        assert doc.attributes[1].default_value == ""
        assert doc.attributes[1].doc.startswith("A Server-level attribute")
        assert doc.attributes[2].name == "labels"
        assert doc.attributes[2].type.type_str == "{str: str}"
        assert doc.attributes[2].is_optional is True
        assert doc.attributes[2].default_value == ""
        assert doc.attributes[2].doc.startswith("A Server-level attribute")
        assert doc.examples.startswith("myCustomApp = AppConfiguration")

    def test_no_type_case(self) -> None:
        docs = resolve(_DIR_PATH / "no_type.k")
        assert len(docs) == 1
        doc = docs[0]
        assert doc.doc.startswith(
            "Server is the common user interface for long-running"
        )
        assert doc.attributes[0].name == "workloadType"
        assert doc.attributes[0].type.type_str == "str"
        assert doc.attributes[0].is_optional is False
        assert doc.attributes[0].default_value == "Deployment"
        assert doc.attributes[0].doc.startswith(
            "Use this attribute to specify which kind of long-running service you want"
        )
        assert doc.attributes[1].name == "name"
        assert doc.attributes[1].type.type_str == "str"
        assert doc.attributes[1].is_optional is False
        assert doc.attributes[1].default_value == ""
        assert doc.attributes[1].doc.startswith("A Server-level attribute")
        assert doc.attributes[2].name == "labels"
        assert doc.attributes[2].type.type_str == "{str: str}"
        assert doc.attributes[2].is_optional is True
        assert doc.attributes[2].default_value == ""
        assert doc.attributes[2].doc.startswith("A Server-level attribute")
        assert doc.attributes[3].name == "replica"
        assert doc.attributes[3].type.type_str == "int"
        assert doc.attributes[3].is_optional is True
        assert doc.attributes[3].default_value == "1"
        assert doc.attributes[3].doc.startswith("Replica of the server")
        assert doc.examples.startswith("myCustomApp = AppConfiguration")

    def test_compact_type_case(self) -> None:
        docs = resolve(_DIR_PATH / "compact_type.k")
        assert len(docs) == 2
        doc = docs[0]
        assert doc.doc.startswith("Metadata is the base schema of all models")
        assert doc.attributes[0].name == "name"
        assert doc.attributes[0].type.type_str == "str"
        assert doc.attributes[0].is_optional is False
        assert doc.attributes[0].default_value == ""
        assert doc.attributes[0].doc.startswith("The name of the resource.")
        assert doc.attributes[1].name == "labels"
        assert doc.attributes[1].type.type_str == "{str: str}"
        assert doc.attributes[1].is_optional is True
        assert doc.attributes[1].default_value == ""
        assert doc.attributes[1].doc.startswith(
            "Labels is a map of string keys and values"
        )
        assert doc.attributes[2].name == "annotations"
        assert doc.attributes[2].type.type_str == "{str: str}"
        assert doc.attributes[2].is_optional is True
        assert doc.attributes[2].default_value == ""
        assert doc.attributes[2].doc.startswith(
            "Annotations is an unstructured key value map"
        )
        assert doc.attributes[3].name == "namespace"
        assert doc.attributes[3].type.type_str == "str"
        assert doc.attributes[3].is_optional is True
        assert doc.attributes[3].default_value == "default"
        assert doc.attributes[3].doc.startswith(
            "Namespaces are intended for use in environments"
        )
        assert doc.examples == ""

        doc = docs[1]
        assert doc.doc.startswith("AppConfiguration is the common user interface")
        assert doc.attributes[0].name == "workloadType"
        assert doc.attributes[0].type.type_str == "str"
        assert doc.attributes[0].is_optional is False
        assert doc.attributes[0].default_value == "Deployment"
        assert doc.attributes[0].doc.startswith("Use this attribute to specify")
        assert doc.attributes[1].name == "name"
        assert doc.attributes[1].type.type_str == "str"
        assert doc.attributes[1].is_optional is False
        assert doc.attributes[1].default_value == ""
        assert doc.attributes[1].doc.startswith("Required.\nA Server-level attribute.")
        assert doc.attributes[2].name == "namespace"
        assert doc.attributes[2].type.type_str == "str"
        assert doc.attributes[2].is_optional is False
        assert doc.attributes[2].default_value == "default"
        assert doc.attributes[2].doc.startswith("Required.\nA Server-level attribute.")
        assert doc.attributes[3].name == "app"
        assert doc.attributes[3].type.type_str == "str"
        assert doc.attributes[3].is_optional is False
        assert doc.attributes[3].default_value == ""
        assert doc.attributes[3].doc.startswith("A Server-level attribute.")
        assert doc.examples.startswith("myCustomApp = AppConfiguration {")

    def test_base_schema_case(self) -> None:
        docs = resolve(_DIR_PATH / "base_schema.k")
        assert len(docs) == 2
        doc = docs[0]
        assert doc.attributes[0].name == "name"
        assert doc.attributes[0].type.type_str == "str"
        assert doc.attributes[0].is_optional is False
        assert doc.attributes[0].default_value == ""
        assert doc.attributes[1].name == "age"
        assert doc.attributes[1].type.type_str == "int"
        assert doc.attributes[1].is_optional is False
        assert doc.attributes[1].default_value == ""

        doc = docs[1]
        assert doc.doc.startswith("Student is the person with a grade")
        assert doc.attributes[0].name == "grade"
        assert doc.attributes[0].type.type_str == "int"
        assert doc.attributes[0].is_optional is False
        assert doc.attributes[0].default_value == "Undefined"
        assert doc.attributes[0].doc.startswith(
            "the current grade that the student is in."
        )


if __name__ == "__main__":
    unittest.main(verbosity=2)
