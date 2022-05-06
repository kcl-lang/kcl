import unittest
import pathlib
import shutil
import filecmp

from kclvm.tools.docs.doc import kcl_doc_generate
import kclvm.tools.docs.formats as doc_formats

_DIR_PATH = pathlib.Path(__file__).parent.joinpath("doc_data")
_SOURCE_PATH = _DIR_PATH / "source_files"
_DOCS_PATH = _DIR_PATH / "docs"
_I18N_PATH = _DIR_PATH / "i18n_inputs"


class KCLDocGenTestData:
    def __init__(
        self,
        filename: str,
        recursive: bool,
        format: str,
        locale: str = "en",
        i18n_path: str = None,
    ):
        self.filename: str = filename
        self.recursive: bool = recursive
        self.format: str = format
        self.locale: str = locale
        self.i18n_path: str = i18n_path


def read_file_content(path) -> str:
    with open(path, "r") as f:
        return f.read()


class KCLDocGenerateTest(unittest.TestCase):
    test_cases = [
        KCLDocGenTestData(
            filename="import_pkg",
            recursive=True,
            format="markdown",
        ),
        KCLDocGenTestData(
            filename="base_schema_pkg",
            recursive=True,
            format="markdown",
        ),
        KCLDocGenTestData(
            filename="config_map",
            recursive=True,
            format="markdown",
        ),
    ]

    def test_doc_gen(self) -> None:
        # make tmp output dir
        tmp_output = _DIR_PATH / "tmp"
        for t_case in self.test_cases:
            tmp_output_current = (
                tmp_output
                / f"docs_{t_case.format}_{t_case.filename.rsplit('.k', 1)[0]}_{t_case.locale}"
            )
            expect_output_current = (
                _DOCS_PATH
                / f"docs_{t_case.format}_{t_case.filename.rsplit('.k', 1)[0]}_{t_case.locale}"
            )
            # generate docs to tmp output dir
            kcl_doc_generate(
                kcl_files=[str(_SOURCE_PATH / t_case.filename)],
                recursively=t_case.recursive,
                output=str(tmp_output_current),
                # output=str(expect_output_current), # for expect docs generate
                format=t_case.format,
                repo_url="https://url/to/source_code",
                with_locale_suffix=True,
            )
            # compare docs between expect and got
            match, mismatch, errors = filecmp.cmpfiles(
                expect_output_current,
                tmp_output_current,
                common=[
                    str(f.relative_to(tmp_output_current))
                    for f in list(
                        tmp_output_current.rglob(
                            f"*{doc_formats.KCLDocSuffix.TO_SUFFIX[t_case.format.upper()]}"
                        )
                    )
                ],
            )
            assert len(mismatch) == 0, f"mismatch exists: {mismatch}. {t_case.filename}"
            assert len(errors) == 0, f"errors exists: {errors}. {t_case.filename}"

            # clear tmp files
            shutil.rmtree(tmp_output)


class KCLDocI18nGenTest(unittest.TestCase):
    test_cases = [
        KCLDocGenTestData(
            filename="simple.k",
            format="markdown",
            recursive=True,
            locale="zh_cn",
            i18n_path="i18n_simple_zh_cn.yaml",
        ),
        KCLDocGenTestData(
            filename="frontend",
            format="markdown",
            recursive=True,
            locale="zh_cn",
            i18n_path="frontend",
        ),
    ]

    def test_doc_gen_by_i18n(self) -> None:
        # make tmp output dir
        tmp_output = _DIR_PATH / "tmp"
        for t_case in self.test_cases:
            tmp_output_current = (
                tmp_output
                / f"docs_{t_case.format}_{t_case.filename.rsplit('.k', 1)[0]}_{t_case.locale}"
            )
            expect_output_current = (
                _DOCS_PATH
                / f"docs_{t_case.format}_{t_case.filename.rsplit('.k', 1)[0]}_{t_case.locale}"
            )
            # generate docs to tmp output dir
            kcl_doc_generate(
                kcl_files=[str(_SOURCE_PATH / t_case.filename)],
                recursively=t_case.recursive,
                output=str(tmp_output_current),
                # output=str(expect_output_current),  # for expect docs generate
                format=t_case.format,
                repo_url="https://url/to/source_code",
                locale=t_case.locale,
                i18n_path=str(_I18N_PATH / t_case.i18n_path),
                with_locale_suffix=True,
            )
            # compare docs between expect and got
            match, mismatch, errors = filecmp.cmpfiles(
                expect_output_current,
                tmp_output_current,
                common=[
                    str(f.relative_to(tmp_output_current))
                    for f in list(
                        tmp_output_current.rglob(
                            f"*{doc_formats.KCLDocSuffix.TO_SUFFIX[t_case.format.upper()]}"
                        )
                    )
                ],
            )
            assert len(mismatch) == 0, f"mismatch exists: {mismatch}. {t_case.filename}"
            assert len(errors) == 0, f"errors exists: {errors}. {t_case.filename}"

            # clear tmp files
            shutil.rmtree(tmp_output)


if __name__ == "__main__":
    unittest.main(verbosity=2)
