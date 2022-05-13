"""Module doc extracts source code documentation from a KCL AST
"""
import io
import os

from typing import Union, List, Dict, Optional
from pathlib import Path

import kclvm.kcl.error as kcl_error
import kclvm.compiler.vfs as vfs
import kclvm.compiler.parser.parser as parser
import kclvm.tools.docs.formats as doc_formats
import kclvm.kcl.types.checker as type_checker
import kclvm.api.object as obj_pkg
import kclvm.kcl.ast as ast
import kclvm.tools.docs.model_pb2 as model
import kclvm.tools.docs.writer as writer
import kclvm.tools.docs.reader as reader
import kclvm.tools.docs.link_resolver as link_resolver
import kclvm.tools.docs.doc_escaper as doc_escaper
import kclvm.tools.docs.doc_parser as doc_parser
import kclvm.tools.docs.i18n as i18n
import kclvm.tools.docs.utils as utils

# ---------------------------------------------------
# Constants
# ---------------------------------------------------

KCL_SUFFIX = "*.k"
INVALID_OUTPUT_FORMAT_MSG = "invalid output format, expected: yaml, json or markdown"
INVALID_I18N_FORMAT_MSG = "invalid i18n metadata format, expected: yaml, json"

# ---------------------------------------------------
# User interface functions used by kcl-doc cli
# ---------------------------------------------------


def kcl_doc_generate(
    kcl_files: List[str],
    output: str,
    format: str = doc_formats.KCLDocFormat.MARKDOWN,
    locale: str = "en",
    recursively=False,
    repo_url: str = None,
    i18n_path: str = None,
    with_locale_suffix: bool = False,
) -> None:
    """
    generate a displayable doc file of a kcl file
    :param kcl_files: the kcl file paths to generate doc on
    :param output: the dir path to output the generated doc file
    :param format: the document format to generate
    :param locale: the document locale to generate
    :param recursively: if search for the kcl files to generate doc on recursively
    :param repo_url: the url to the source code repo
    :param i18n_path: the i18n input path
    """
    # check if the format and locale is valid
    locale = locale.lower()
    format_upper = format.upper()
    if format_upper not in doc_formats.KCLDocFormat.MAPPING:
        kcl_error.report_exception(
            err_type=kcl_error.ErrType.CompileError_TYPE,
            arg_msg=INVALID_OUTPUT_FORMAT_MSG,
        )
    i18n.check_locale(locale)
    # check if all the files exist
    if len(kcl_files) == 0:
        kcl_error.report_exception(
            err_type=kcl_error.ErrType.CannotFindModule_TYPE,
            arg_msg="Empty list of input KCL files",
        )
    for kcl_file in kcl_files:
        if not Path(kcl_file).exists():
            kcl_error.report_exception(
                err_type=kcl_error.ErrType.CannotFindModule_TYPE,
                arg_msg=f"Cannot find the kcl file, please check whether the file path {kcl_file} exists"
                if kcl_file
                else f"The file path {kcl_file} is None",
            )
    # parse module docs
    module_docs = _parse_file_docs(
        kcl_files, format_upper, locale, recursively, repo_url, with_locale_suffix
    )
    # write to output
    path = Path(output).resolve()
    for _, doc in module_docs.items():
        # if the i18n file exists, use the doc content decoded from the existing file
        i18n_file_path, i18n_format = _existed_i18n_file_path(
            i18n_path, path, doc.name, locale
        )
        if i18n_file_path and i18n_format:
            doc = _read_from_file(i18n_format, i18n_file_path)
            _escape_text({doc.name: doc}, format_upper)
        doc_file_path = _doc_file_path(
            path / Path(doc.relative_path).parent,
            doc.name,
            locale,
            format_upper,
            with_locale_suffix,
        )
        _write_to_file(doc, format_upper, doc_file_path)


def kcl_i18n_init(
    kcl_files: List[str],
    output: str,
    format: str,
    locale: str = "en",
    recursively=False,
    with_locale_suffix: bool = False,
) -> None:
    """
    init an i18n source doc file of a KCL file. Users can then modify the document part of it and generate docs in other formats based on it
    :param kcl_files: the kcl file paths to generate doc on
    :param output: the dir path to output the inited i18n file
    :param format: the document format to init
    :param locale: the document locale to init
    :param recursively: if search for the kcl files to generate doc on recursively
    """
    # check if the format and locale is valid
    locale = locale.lower()
    format_upper = format.upper()
    if format_upper not in doc_formats.KCLI18NFormat.MAPPING:
        kcl_error.report_exception(
            err_type=kcl_error.ErrType.CompileError_TYPE,
            arg_msg=INVALID_I18N_FORMAT_MSG,
        )
    i18n.check_locale(locale)
    # check if all the files exist
    if len(kcl_files) == 0:
        kcl_error.report_exception(
            err_type=kcl_error.ErrType.CannotFindModule_TYPE,
            arg_msg="Empty list of input KCL files",
        )
    for kcl_file in kcl_files:
        if not Path(kcl_file).exists():
            kcl_error.report_exception(
                err_type=kcl_error.ErrType.CannotFindModule_TYPE,
                arg_msg=f"Cannot find the kcl file, please check whether the file path {kcl_file} exists"
                if kcl_file
                else f"The file path {kcl_file} is None",
            )
    # parse module docs
    module_docs = _parse_file_docs(
        kcl_files,
        format_upper,
        locale,
        recursively,
        with_locale_suffix=with_locale_suffix,
    )
    # write to output
    path = Path(output).resolve()
    path.mkdir(parents=True, exist_ok=True)
    for _, doc in module_docs.items():
        i18n_path = _i18n_file_path(path, doc.name, locale, format_upper)
        # reset doc content before gen i18n file
        _write_to_file(doc, format_upper, i18n_path)


def kcl_i18n_info(kcl_files: List[str], format: str, locale: str) -> None:
    """
    show an i18n source doc of a kcl file
    :param kcl_files: the kcl files to show i18n info
    :param format: the i18n file format to display
    :param locale: the i18n file locale to display
    """
    # check if the format and locale is valid
    locale = locale.lower()
    format_upper = format.upper()
    if format_upper not in doc_formats.KCLI18NFormat.MAPPING:
        kcl_error.report_exception(
            err_type=kcl_error.ErrType.CompileError_TYPE,
            arg_msg=INVALID_I18N_FORMAT_MSG,
        )
    i18n.check_locale(locale)

    for kcl_file in kcl_files:
        # calculate the i18n file path
        path = _i18n_file_path(
            Path(kcl_file).parent, utils.module_name(kcl_file), locale, format_upper
        )
        # display the doc content read from i18n file to console
        print(f"i18n file: {path} \ncontent:")
        path = Path(path)
        if path.exists():
            module_doc = _read_from_file(format_upper, path)
            if module_doc:
                output = io.StringIO()
                _write_to_io(module_doc, format_upper, output)
                print(output.getvalue())
            else:
                print("Failed to parse i18n locale file, get nothing.")
        else:
            print(
                f"I18n local file not exist. \n    KCL file path: {kcl_file}, format: {format}, locale: {locale}"
            )


# ---------------------------------------------------
# Internal functions
# ---------------------------------------------------


def _read_from_file(format: str, in_file: Union[str, Path]) -> model.ModuleDoc:
    with Path(in_file).open(mode="r", encoding="utf-8") as in_io:
        module_doc = _read_from_io(format, in_io)
    return module_doc


def _read_from_io(format: str, in_io: io.TextIOBase) -> model.ModuleDoc:
    doc_reader = reader.factory.get(format, in_io)
    return doc_reader.read_doc()


def _write_to_file(
    doc: model.ModuleDoc, format: str, out_file: Union[str, Path]
) -> None:
    Path(out_file).parent.mkdir(parents=True, exist_ok=True)
    with Path(out_file).open(mode="w", encoding="utf-8") as out:
        _write_to_io(doc, format, out)


def _write_to_io(doc: model.ModuleDoc, format: str, out: io.TextIOBase) -> None:
    doc_writer = writer.factory.get(format, out)
    doc_writer.write_doc(doc)


def _doc_file_name(
    module_name: str, locale: str, format: str, with_locale_suffix: bool = False
) -> str:
    return (
        f"doc_{module_name}_{locale}" + doc_formats.KCLDocSuffix.TO_SUFFIX[format]
        if with_locale_suffix
        else f"doc_{module_name}{doc_formats.KCLDocSuffix.TO_SUFFIX[format]}"
    )


def _doc_file_path(
    path: Path, name: str, locale: str, format: str, with_locale_suffix: bool = False
) -> Path:
    return path / _doc_file_name(
        module_name=name,
        locale=locale,
        format=format,
        with_locale_suffix=with_locale_suffix,
    )


def _i18n_file_path(path: Path, name: str, locale: str, format: str) -> Path:
    return path / (f"i18n_{name}_{locale}" + doc_formats.KCLDocSuffix.TO_SUFFIX[format])


def _existed_i18n_file_path(
    i18n_path: str, path: Path, name: str, locale: str
) -> (Optional[Path], Optional[str]):
    """
    check if there are existed i18n files for current source file path.
    When there exists more than one possible i18n files, the priority to return is: KCL -> YAML -> JSON

    :param i18n_path: the i18n input file path
    :param path: the dir path that contains KCL source file
    :param name: the KCL module name to generate doc on
    :param locale: the target locale
    :return: the i18n file path if existed, else return None.
    """
    if i18n_path:
        i18n_path = Path(i18n_path)
        if not i18n_path.exists():
            return None, None
        if (
            i18n_path.is_file()
            and i18n_path.suffix in doc_formats.KCLI18NFormat.FROM_SUFFIX
        ):
            return i18n_path, doc_formats.KCLI18NFormat.FROM_SUFFIX[i18n_path.suffix]
        elif i18n_path.is_dir():
            path = i18n_path
        else:
            return None, None
    yaml_i18n_path = _i18n_file_path(path, name, locale, doc_formats.KCLI18NFormat.YAML)
    if yaml_i18n_path.exists():
        return yaml_i18n_path, doc_formats.KCLI18NFormat.YAML

    json_i18n_path = _i18n_file_path(path, name, locale, doc_formats.KCLI18NFormat.JSON)
    if json_i18n_path.exists():
        return json_i18n_path, doc_formats.KCLI18NFormat.JSON

    return None, None


def _escape_text(module_docs: Dict[str, model.ModuleDoc], format: str):
    escaper = doc_escaper.factory.get(format)
    for path, module in module_docs.items():
        module_docs[path] = escaper.escape(module)


def _resolve_link(
    root: str,
    module_docs: Dict[str, model.ModuleDoc],
    format: str,
    locale: str,
    with_locale_suffix: bool = False,
):
    resolver = link_resolver.factory.get(format, _doc_file_name)
    for path, module in module_docs.items():
        module_docs[path] = resolver.resolve(module, root, locale, with_locale_suffix)


def _parse_file_docs(
    kcl_files: List[str],
    format: str,
    locale: str,
    recursively: bool,
    repo_url: str = None,
    with_locale_suffix: bool = False,
) -> Dict[str, model.ModuleDoc]:
    pkgs: List[str] = []
    if recursively:
        for kcl_file in kcl_files:
            pkgs.extend(_find_kcl_pkgs_recursively(Path(kcl_file)))
        pkgs = list(set(pkgs))
    else:
        pkgs = kcl_files
    if len(pkgs) == 0:
        return {}
    root: str = vfs.GetPkgRoot(pkgs[0]) or os.path.dirname(pkgs[0])
    module_docs: Dict[str, model.ModuleDoc] = {}
    trigger_lines = [
        f"import .{str(Path(pkg).relative_to(root)).replace('/', '.').rstrip('.k')}\n"
        for pkg in pkgs
    ]
    trigger_file = Path(root) / "trigger_doc_gen.k"
    with open(trigger_file, "w") as f:
        f.writelines(trigger_lines)
    try:
        prog = parser.LoadProgram(
            str(trigger_file),
            work_dir=root,
        )
        type_checker.ResolveProgramImport(prog)
        checker = type_checker.TypeChecker(prog, type_checker.CheckConfig())
        checker.check_import(prog.MAIN_PKGPATH)
        checker.init_global_types()
        del prog.pkgs[prog.MAIN_PKGPATH]
    finally:
        os.remove(trigger_file)
    pkg_docs = _parse_program_docs(prog, checker, repo_url)
    _escape_text(pkg_docs, format)
    _resolve_link(prog.root, pkg_docs, format, locale, with_locale_suffix)

    module_docs.update(pkg_docs)
    return module_docs


def _parse_program_docs(
    program: ast.Program, checker: type_checker.TypeChecker, repo_url: str = None
) -> Dict[str, model.ModuleDoc]:

    pkgs: Dict[str, List[ast.Module]] = program.pkgs
    module_docs: Dict[str, model.ModuleDoc] = {}
    for pkgpath, modules in pkgs.items():
        for m in modules:
            schema_docs: list = []
            schema_list = m.GetSchemaList()
            # only generate module doc if the module contains schemas
            if schema_list:
                for schema in schema_list:
                    current_scope = checker.scope_map[pkgpath]
                    schema_obj_type = current_scope.elems[schema.name].type
                    if isinstance(schema_obj_type, obj_pkg.KCLSchemaDefTypeObject):
                        schema_doc = doc_parser.SchemaDocParser(
                            schema=schema,
                            schema_type=schema_obj_type.schema_type,
                            root=program.root,
                        ).doc
                        schema_docs.append(schema_doc)
                module_doc = model.ModuleDoc(
                    name=utils.module_name(m.filename),
                    relative_path=m.relative_filename,
                    doc=m.doc,
                    schemas=schema_docs,
                    source_code_url=repo_url,
                )
                module_docs[m.relative_filename] = module_doc
    return module_docs


def _find_kcl_pkgs_recursively(file: Path) -> List[str]:
    if file.is_file():
        return [str(file)]
    all_files = file.rglob("*.k")
    pkgs: List[str] = [str(f.parent) for f in all_files]
    return list(set(pkgs))
