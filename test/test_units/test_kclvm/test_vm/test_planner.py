# Copyright 2021 The KCL Authors. All rights reserved.

import unittest

import kclvm.api.object as objpkg
import kclvm.vm.planner as planner


class PlannerTest(unittest.TestCase):
    def test_object_planner(self):
        cases = [
            {"obj": {"key": 1}, "expected": {"key": 1}},
            {
                "obj": {"key": [1, 2, 3]},
                "expected": {"key": [1, 2, 3]},
            },
            {
                "obj": {"key": True},
                "expected": {"key": True},
            },
            {
                "obj": {"key": {"key": 1}},
                "expected": {"key": {"key": 1}},
            },
        ]
        for case in cases:
            obj, expected = case["obj"], case["expected"]
            self.assertEqual(
                planner.ObjectPlanner().to_python(obj),
                expected,
                msg=f"{obj}",
            )
            self.assertEqual(
                planner.ObjectPlanner().to_python(objpkg.to_kcl_obj(obj)),
                expected,
                msg=f"{obj}",
            )
            self.assertEqual(
                planner.ObjectPlanner().to_python(objpkg.to_kcl_obj(obj).value),
                expected,
                msg=f"{obj}",
            )


if __name__ == "__main__":
    unittest.main(verbosity=2)
