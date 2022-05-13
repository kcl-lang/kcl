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


class KCLDocCheckerTest(unittest.TestCase):
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


if __name__ == "__main__":
    unittest.main(verbosity=2)
