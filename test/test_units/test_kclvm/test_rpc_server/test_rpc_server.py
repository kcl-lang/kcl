# Copyright 2020 The KCL Authors. All rights reserved.

import time
import json
import pathlib
import unittest
import subprocess
import requests
import os

import kclvm.internal.util as util


SERVER_CMD = "kclvm -m kclvm.program.rpc-server -http=127.0.0.1:2021"
HEADERS = {
    "content-type": "accept/json",
}


class TestRpcServer(unittest.TestCase):
    def setUp(self):
        self.p = subprocess.Popen(SERVER_CMD.split())
        # Sleep enough time to establish connection
        time.sleep(10)
        super().setUp()

    def tearDown(self):
        self.p.kill()
        super().tearDown()

    def test_rpc_server_normal(self):
        apis = [
            "EvalCode",
            "ExecProgram",
            "GetSchemaType",
            "ResolveCode",
            "SpliceCode",
            "ValidateCode",
            "ValidateCode",
            "ValidateCode",
        ]
        data_files = [
            "eval-code",
            "exec-program",
            "get-schema",
            "resolve-code",
            "splice-code",
            "vet-hello",
            "vet-simple",
            "vet-single",
        ]
        test_path = "testdata"
        for api, data_file in zip(apis, data_files):
            try:
                json_file = pathlib.Path(__file__).parent.joinpath(
                    f"{test_path}/{data_file}.json"
                )
                json_data = json_file.read_text(encoding="utf-8")
                res = requests.post(
                    f"http://127.0.0.1:2021/api:protorpc/KclvmService.{api}",
                    data=json_data.encode("utf-8"),
                    headers=HEADERS,
                )
                res_data = res.json()
                self.assertEqual(
                    res_data["error"], "", msg=f"api: {api}, data_file: {data_file}"
                )
            except Exception as err:
                self.assertTrue(False, msg=f"api: {api}, data_file: {data_file}, reason: {err}")
                continue

    def test_rpc_server_invalid(self):
        apis = [
            "EvalCode",
            "ExecProgram",
            "GetSchemaType",
            "ResolveCode",
            "SpliceCode",
            "ValidateCode",
        ]
        data_files = [
            "eval-code",
            "exec-program",
            "get-schema",
            "resolve-code",
            "splice-code",
            "vet-simple",
        ]
        test_path = "invalid_testdata"
        for api, data_file in zip(apis, data_files):
            try:
                json_file = pathlib.Path(__file__).parent.joinpath(
                    f"{test_path}/{data_file}.json"
                )
                json_data = json_file.read_text(encoding="utf-8")
                res = requests.post(
                    f"http://127.0.0.1:2021/api:protorpc/KclvmService.{api}",
                    data=json_data.encode("utf-8"),
                    headers=HEADERS,
                )
                res_data = res.json()
                self.assertTrue(
                    bool(res_data["error"]), msg=f"api: {api}, data_file: {data_file}"
                )
            except:
                self.assertTrue(False, msg=f"api: {api}, data_file: {data_file}")
                continue

    # TODO: enable this test after the kcl-go upgraded
    def _test_rpc_server_ListDepFiles(self):
        appWorkDir = pathlib.Path(__file__).parent.joinpath(f"testdata/kcl-module/app0")

        api = "ListDepFiles"
        json_data = f"{{\"work_dir\": \"{appWorkDir}\"}}"
        res = requests.post(
            f"http://127.0.0.1:2021/api:protorpc/KclvmService.{api}",
            data=json_data.encode("utf-8"),
            headers=HEADERS,
        )

        res_data = res.json()
        self.assertEqual(res_data["error"], "", msg=f"api: {api}, res_data: {res_data}, appWorkDir: {appWorkDir}")
        self.assertTrue(res_data["result"], msg=f"api: {api}, res_data: {res_data}, appWorkDir: {appWorkDir}")

        expect = [
            "main.k",
            "app0/before/base.k",
            "app0/main.k",
            "app0/sub/sub.k",
        ]

        self.assertEqual(
            sorted(res_data["result"]["files"]),
            sorted(expect),
            msg=f"api: {api}, res_data: {res_data}, expect: {expect}, appWorkDir: {appWorkDir}"
        )


if __name__ == "__main__":
    unittest.main(verbosity=2)
