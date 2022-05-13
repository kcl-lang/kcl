# Copyright 2021 The KCL Authors. All rights reserved.

import pathlib
import unittest
import json
from pathlib import Path
import pygls.lsp.types.basic_structures as pygls_basic
from kclvm.internal.gpyrpc.gpyrpc_pb2 import Position
import kclvm.tools.langserver.grpc_wrapper as wrapper

_TEST_PATH_NAME = "test_data"
_GO_TO_DEF_PATH_NAME = "go_to_def"
_COMPLETE_PATH_NAME = "complete"
_DOCUMENT_SYMBOL_PATH_NAME = "document_symbol"
_HOVER_PATH_NAME = "hover"
_DIR_PATH = pathlib.Path(__file__).parent.joinpath(_TEST_PATH_NAME)


class RequestWrapperTest(unittest.TestCase):
    def test_go_to_def_wrapper(self):
        filename = str(_DIR_PATH / _GO_TO_DEF_PATH_NAME / "simple.k")
        got_result = wrapper.go_to_def_wrapper(
            Position(line=1, column=4, filename=filename),
        )
        expect_result = (
                '[{"uri": "'
                + filename
                + '", "range": {"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 1}}}]'
        )
        self.assertEqual(
            expect_result, got_result, f"expect: {expect_result}, got: {got_result}"
        )

    def test_complete_wrapper(self):
        filename = str(_DIR_PATH / _COMPLETE_PATH_NAME / "simple.k")
        got_result = wrapper.complete_wrapper(
            pos=Position(line=1, column=5, filename=filename),
            name="a",
        )
        expect_result = '[{"label": "aa", "kind": 12, "tags": null, "detail": null, "documentation": null, "deprecated": false, "preselect": false, "sort_text": null, "filter_text": null, "insert_text": null, "insert_text_format": null, "text_edit": null, "additional_text_edits": null, "commit_characters": null, "command": null, "data": null}, {"label": "abs", "kind": 3, "tags": null, "detail": null, "documentation": null, "deprecated": false, "preselect": false, "sort_text": null, "filter_text": null, "insert_text": null, "insert_text_format": null, "text_edit": null, "additional_text_edits": null, "commit_characters": null, "command": null, "data": null}, {"label": "all_true", "kind": 3, "tags": null, "detail": null, "documentation": null, "deprecated": false, "preselect": false, "sort_text": null, "filter_text": null, "insert_text": null, "insert_text_format": null, "text_edit": null, "additional_text_edits": null, "commit_characters": null, "command": null, "data": null}, {"label": "any_true", "kind": 3, "tags": null, "detail": null, "documentation": null, "deprecated": false, "preselect": false, "sort_text": null, "filter_text": null, "insert_text": null, "insert_text_format": null, "text_edit": null, "additional_text_edits": null, "commit_characters": null, "command": null, "data": null}]'
        self.assertEqual(
            expect_result, got_result, f"expect: {expect_result}, got: {got_result}"
        )

    def test_document_symbol_wrapper(self):
        filename = str(_DIR_PATH / _DOCUMENT_SYMBOL_PATH_NAME / "symbol.k")
        got_result = wrapper.document_symbol_wrapper(filename)
        expect_result = '[{"name": "a", "kind": 13, "range": {"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 9}}, "selectionRange": {"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 1}}, "detail": null, "children": null, "deprecated": false}, {"name": "b", "kind": 13, "range": {"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 9}}, "selectionRange": {"start": {"line": 0, "character": 4}, "end": {"line": 0, "character": 5}}, "detail": null, "children": null, "deprecated": false}, {"name": "Person", "kind": 23, "range": {"start": {"line": 2, "character": 0}, "end": {"line": 7, "character": 20}}, "selectionRange": {"start": {"line": 2, "character": 7}, "end": {"line": 2, "character": 13}}, "detail": null, "children": [{"name": "mixin", "kind": 7, "range": {"start": {"line": 3, "character": 4}, "end": {"line": 3, "character": 9}}, "selectionRange": {"start": {"line": 3, "character": 4}, "end": {"line": 3, "character": 9}}, "detail": null, "children": [{"name": "aMixin", "kind": 13, "range": {"start": {"line": 4, "character": 8}, "end": {"line": 4, "character": 14}}, "selectionRange": {"start": {"line": 4, "character": 8}, "end": {"line": 4, "character": 14}}, "detail": null, "children": null, "deprecated": false}], "deprecated": false}, {"name": "age", "kind": 7, "range": {"start": {"line": 6, "character": 4}, "end": {"line": 6, "character": 17}}, "selectionRange": {"start": {"line": 6, "character": 4}, "end": {"line": 6, "character": 7}}, "detail": null, "children": null, "deprecated": false}, {"name": "name", "kind": 7, "range": {"start": {"line": 7, "character": 4}, "end": {"line": 7, "character": 20}}, "selectionRange": {"start": {"line": 7, "character": 4}, "end": {"line": 7, "character": 8}}, "detail": null, "children": null, "deprecated": false}], "deprecated": false}, {"name": "person", "kind": 13, "range": {"start": {"line": 9, "character": 0}, "end": {"line": 11, "character": 1}}, "selectionRange": {"start": {"line": 9, "character": 0}, "end": {"line": 9, "character": 6}}, "detail": null, "children": null, "deprecated": false}]'
        self.assertEqual(
            expect_result, got_result, f"expect: {expect_result}, got: {got_result}"
        )

    def test_hover_wrapper(self):
        filename = str(_DIR_PATH / _HOVER_PATH_NAME / "hello.k")
        got_result = wrapper.hover_wrapper(
            pos=Position(line=7, column=4, filename=filename),
            code=Path(filename).read_text(encoding="utf-8")
        )
        expect = {
            "contents": {
                "kind": "plaintext",
                "value": f"image\ntype: str\ndefined in:{filename}"
            },
            "range": pygls_basic.Range(
                start=pygls_basic.Position(line=7, character=4),
                end=pygls_basic.Position(line=7, character=9),
            ),
        }
        expect_result = json.dumps(obj=expect, default=lambda x: x.__dict__)
        self.assertEqual(
            expect_result, got_result, f"expect: {expect_result}, got: {got_result}"
        )


if __name__ == "__main__":
    unittest.main(verbosity=2)
