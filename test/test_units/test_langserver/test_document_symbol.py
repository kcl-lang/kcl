# Copyright 2021 The KCL Authors. All rights reserved.

import unittest
import pathlib

from pygls.lsp.types.basic_structures import Range, Position
from pygls.lsp.types.language_features.document_symbol import DocumentSymbol, SymbolKind

import kclvm.tools.langserver.document_symbol as symbol

_DOCUMENT_SYMBOL_DIR = pathlib.Path(__file__).parent.joinpath(
    "test_data/document_symbol"
)


class DocumentSymbolTest(unittest.TestCase):
    def test_file_to_symbol(self):
        expect_result = [
            DocumentSymbol(
                name="a",
                kind=SymbolKind.Variable,
                range=Range(
                    start=Position(line=0, character=0),
                    end=Position(line=0, character=9),
                ),
                selection_range=Range(
                    start=Position(line=0, character=0),
                    end=Position(line=0, character=1),
                ),
            ),
            DocumentSymbol(
                name="b",
                kind=SymbolKind.Variable,
                range=Range(
                    start=Position(line=0, character=0),
                    end=Position(line=0, character=9),
                ),
                selection_range=Range(
                    start=Position(line=0, character=4),
                    end=Position(line=0, character=5),
                ),
            ),
            DocumentSymbol(
                name="Person",
                kind=SymbolKind.Struct,
                range=Range(
                    start=Position(line=2, character=0),
                    end=Position(line=7, character=20),
                ),
                selection_range=Range(
                    start=Position(line=2, character=7),
                    end=Position(line=2, character=13),
                ),
                children=[
                    DocumentSymbol(
                        name="mixin",
                        kind=SymbolKind.Property,
                        range=Range(
                            start=Position(line=3, character=4),
                            end=Position(line=3, character=9),
                        ),
                        selection_range=Range(
                            start=Position(line=3, character=4),
                            end=Position(line=3, character=9),
                        ),
                        children=[
                            DocumentSymbol(
                                name="aMixin",
                                kind=SymbolKind.Variable,
                                range=Range(
                                    start=Position(line=4, character=8),
                                    end=Position(line=4, character=14),
                                ),
                                selection_range=Range(
                                    start=Position(line=4, character=8),
                                    end=Position(line=4, character=14),
                                ),
                            )
                        ],
                    ),
                    DocumentSymbol(
                        name="age",
                        kind=SymbolKind.Property,
                        range=Range(
                            start=Position(line=6, character=4),
                            end=Position(line=6, character=17),
                        ),
                        selection_range=Range(
                            start=Position(line=6, character=4),
                            end=Position(line=6, character=7),
                        ),
                    ),
                    DocumentSymbol(
                        name="name",
                        kind=SymbolKind.Property,
                        range=Range(
                            start=Position(line=7, character=4),
                            end=Position(line=7, character=20),
                        ),
                        selection_range=Range(
                            start=Position(line=7, character=4),
                            end=Position(line=7, character=8),
                        ),
                    ),
                ],
            ),
            DocumentSymbol(
                name="person",
                kind=SymbolKind.Variable,
                range=Range(
                    start=Position(line=9, character=0),
                    end=Position(line=11, character=1),
                ),
                selection_range=Range(
                    start=Position(line=9, character=0),
                    end=Position(line=9, character=6),
                ),
            ),
        ]
        symbols = symbol.document_symbol(_DOCUMENT_SYMBOL_DIR.joinpath("symbol.k"))
        self.check_result(expect_result, symbols)

    def test_invalid_grammar(self):
        symbols = symbol.document_symbol(
            _DOCUMENT_SYMBOL_DIR.joinpath("invalid_grammar.k")
        )
        assert (
            not symbols
        ), "invalid grammar should got empty document symbol result for now"

    def test_invalid_semantic(self):
        expect_result = [
            DocumentSymbol(
                name="a",
                kind=SymbolKind.Variable,
                range=Range(
                    start=Position(line=0, character=0),
                    end=Position(line=0, character=5),
                ),
                selection_range=Range(
                    start=Position(line=0, character=0),
                    end=Position(line=0, character=1),
                ),
            ),
            DocumentSymbol(
                name="c",
                kind=SymbolKind.Variable,
                range=Range(
                    start=Position(line=1, character=0),
                    end=Position(line=1, character=5),
                ),
                selection_range=Range(
                    start=Position(line=1, character=0),
                    end=Position(line=1, character=1),
                ),
            ),
        ]
        symbols = symbol.document_symbol(
            _DOCUMENT_SYMBOL_DIR.joinpath("invalid_semantic.k")
        )
        self.check_result(expect_result, symbols)

    def check_result(self, expect, got):
        self.assertEqual(
            len(expect),
            len(got),
            f"inconsistent number of symbols found, expect: {len(expect)}, got: {len(got)}",
        )
        for i, s in enumerate(got):
            self.assertEqual(expect[i], s, f"expect: {expect[i]}, got: {s}")


if __name__ == "__main__":
    unittest.main(verbosity=2)
