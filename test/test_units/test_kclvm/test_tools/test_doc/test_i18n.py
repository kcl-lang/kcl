import filecmp
import pathlib
import shutil
import unittest

import kclvm.tools.docs.formats as doc_formats
from kclvm.tools.docs.doc import kcl_i18n_init

_DIR_PATH = pathlib.Path(__file__).parent.joinpath("doc_data")
_SOURCE_PATH = _DIR_PATH / "source_files"
_DOCS_PATH = _DIR_PATH / "docs"


class KCLDocI18nTestData:
    def __init__(self, filename: str, format: str, recursive: bool, locale: str):
        self.filename: str = filename
        self.format: str = format
        self.recursive: bool = recursive
        self.locale: str = locale


class KCLDocI18nInitTest(unittest.TestCase):
    test_cases = [
        KCLDocI18nTestData(
            filename="frontend",
            format=doc_formats.KCLI18NFormat.YAML,
            recursive=True,
            locale="zh_cn",
        ),
        KCLDocI18nTestData(
            filename="simple.k",
            format=doc_formats.KCLI18NFormat.YAML,
            recursive=True,
            locale="zh_cn",
        ),
        KCLDocI18nTestData(
            filename="compact_type.k",
            format=doc_formats.KCLI18NFormat.YAML,
            recursive=True,
            locale="zh_cn",
        ),
    ]

    def test_doc_i18n_init(self) -> None:
        # make tmp output dir
        tmp_output = _DIR_PATH / "tmp"
        for t_case in self.test_cases:
            tmp_output_current = (
                tmp_output
                / f"i18n_docs_{t_case.format}_{t_case.filename.rsplit('.k', 1)[0]}_{t_case.locale}"
            )
            expect_output_current = (
                _DOCS_PATH
                / f"i18n_docs_{t_case.format}_{t_case.filename.rsplit('.k', 1)[0]}_{t_case.locale}"
            )
            # generate docs to tmp output dir
            kcl_i18n_init(
                kcl_files=[str(_SOURCE_PATH / t_case.filename)],
                recursively=t_case.recursive,
                output=str(tmp_output_current),
                # output=str(expect_output_current), # for expect docs generate
                format=t_case.format,
                locale=t_case.locale,
                with_locale_suffix=True,
            )

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
