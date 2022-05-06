# Copyright 2020 The KCL Authors. All rights reserved.

import unittest
import pathlib

import kclvm.config as config


class TestSettingsFile(unittest.TestCase):
    def test_load_settings_files(self):
        files = [
            str(pathlib.Path(__file__).parent.joinpath("test_data/settings.yaml")),
        ]
        work_dir = str(pathlib.Path(__file__).parent.joinpath("test_data"))
        cli_config = config.load_settings_files(work_dir, files)
        expected_files = [
            str(pathlib.Path(__file__).parent.joinpath("test_data/base.k")),
            str(pathlib.Path(__file__).parent.joinpath("test_data/main.k")),
        ]
        self.assertEqual(cli_config.kcl_cli_configs.files, expected_files)
        keys = [opt.key for opt in cli_config.kcl_options]
        values = [opt.value for opt in cli_config.kcl_options]
        self.assertListEqual(keys, ["app", "env-type"])
        self.assertListEqual(values, ["kclvm", "dev"])


if __name__ == "__main__":
    unittest.main(verbosity=2)
