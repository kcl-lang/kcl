# Copyright 2021 The KCL Authors. All rights reserved.

import unittest

import kclvm.api.object as obj
import kclvm.vm as vm

from kclvm.vm.runtime.evaluator import ValueCache, Backtracking

KEY_FIELD = "key"
VALUE_FIELD = "value"


class KCLSchemaLazyEvalTest(unittest.TestCase):
    def test_cache(self):
        """Runtime cache test"""
        values = [
            1,
            1.1,
            "s",
            True,
            False,
            None,
            [],
            {},
            [1, 2, 3],
            {"key": "value"},
        ]
        cases = [
            {KEY_FIELD: KEY_FIELD + str(i), VALUE_FIELD: obj.to_kcl_obj(v)}
            for i, v in enumerate(values)
        ]
        cache = ValueCache()
        for case in cases:
            cache.set(case[KEY_FIELD], case[VALUE_FIELD])
            self.assertEqual(cache.get(case[KEY_FIELD]), case[VALUE_FIELD])

    def test_back_track(self):
        name = "key"
        err_name = "err"
        backtracking = Backtracking()
        self.assertEqual(backtracking.is_backtracking(name), False)
        with backtracking.catch(name):
            level = backtracking.tracking_level(name)
            self.assertEqual(level, 1)
            self.assertEqual(backtracking.is_backtracking(name), True)
            self.assertEqual(backtracking.is_backtracking(err_name), False)
            with backtracking.catch(name):
                level = backtracking.tracking_level(name)
                self.assertEqual(level, 2)
                self.assertEqual(backtracking.is_backtracking(name), True)
                with backtracking.catch(name):
                    level = backtracking.tracking_level(name)
                    self.assertEqual(level, 3)
                    self.assertEqual(backtracking.is_backtracking(name), True)
            with backtracking.catch(name):
                level = backtracking.tracking_level(name)
                self.assertEqual(level, 2)
                self.assertEqual(backtracking.is_backtracking(name), True)
            level = backtracking.tracking_level(name)
            self.assertEqual(level, 1)
            self.assertEqual(backtracking.is_backtracking(name), True)
            self.assertEqual(backtracking.is_backtracking(err_name), False)
        self.assertEqual(backtracking.is_backtracking(name), False)


if __name__ == "__main__":
    unittest.main(verbosity=2)
