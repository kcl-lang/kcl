# Copyright 2020 The KCL Authors. All rights reserved.

import unittest
import pathlib

import kclvm.config as config


class TestConfigSettingFile(unittest.TestCase):
    def test_cli_setting_action(self):
        settings_file = str(pathlib.Path(__file__).parent.joinpath("test_data/settings.yaml"))
        action = config.KCLCLISettingAction()
        keys, values = [], []
        action.deal_setting_file(settings_file, keys, values)
        self.assertListEqual(keys, ["app", "env-type"])
        self.assertListEqual(values, ["kclvm", "dev"])

if __name__ == "__main__":
    unittest.main(verbosity=2)
