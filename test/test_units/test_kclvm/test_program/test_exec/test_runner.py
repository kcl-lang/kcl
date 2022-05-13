# Copyright 2021 The KCL Authors. All rights reserved.

import unittest
import pathlib

import kclvm.program.exec.runner as runner


class KCLCompilerBuildCacheTest(unittest.TestCase):
    def test_schema_infer(self):
        test_path = pathlib.Path(__file__).parent.joinpath("testdata/schema_infer")
        test_main = test_path.joinpath("main.k")
        result = runner.Run([str(test_main)], work_dir=str(test_path))
        self.assertEqual(list(result.filter_by_path_selector().keys()), ['Person', 'x0', 'x1', 'x2', '@pkg'])

    def _test_schema_infer_native(self):
        test_path = pathlib.Path(__file__).parent.joinpath("testdata/schema_infer")
        test_main = test_path.joinpath("main.k")
        result = runner.Run([str(test_main)], work_dir=str(test_path), target="native")
        self.assertEqual(list(result.filter_by_path_selector().keys()), ['Person', 'x0', 'x1', 'x2', '@pkg'])


if __name__ == "__main__":
    unittest.main(verbosity=2)
