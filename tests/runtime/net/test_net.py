# Copyright The KCL Authors. All rights reserved.

import typing
import unittest
import sys
import os

# Add the parent directory to the path to import kclvm_runtime
parent_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.append(parent_dir)
import kclvm_runtime

# https://github.com/python/cpython/blob/main/Lib/test
_Dylib = kclvm_runtime.KclRuntimeDylib()


class kclx_Net:
    def __init__(self, dylib_=None):
        self.dylib = dylib_ if dylib_ else _Dylib
    def is_global_unicast_IP(self, value: str) -> bool:
        return self.dylib.Invoke(f"net.is_global_unicast_IP", value)
    def is_link_local_multicast_IP(self, value: str) -> bool:
        return self.dylib.Invoke(f"net.is_link_local_multicast_IP", value)
    def is_interface_local_multicast_IP(self, value: str) -> bool:
        return self.dylib.Invoke(f"net.is_interface_local_multicast_IP", value)
    def is_multicast_IP(self, value: str) -> bool:
        return self.dylib.Invoke(f"net.is_multicast_IP", value)
    def is_loopback_IP(self, value: str) -> bool:
        return self.dylib.Invoke(f"net.is_loopback_IP", value)
    def is_link_local_unicast_IP(self, value: str) -> bool:
        return self.dylib.Invoke(f"net.is_link_local_unicast_IP", value)
    def is_unspecified_IP(self, value: str) -> bool:
        return self.dylib.Invoke(f"net.is_unspecified_IP", value)


kclxnet = kclx_Net(_Dylib)

class BaseTest(unittest.TestCase):
    def test_is_interface_local_multicast_IP(self):
        self.assertFalse(kclxnet.is_interface_local_multicast_IP("224.0.0.0"))
        self.assertTrue(kclxnet.is_interface_local_multicast_IP("ff11::1"))
    def test_is_link_local_multicast_IP(self):
        self.assertTrue(kclxnet.is_link_local_multicast_IP("ff12::1"))
    def test_is_global_unicast_IP(self):
        self.assertTrue(kclxnet.is_global_unicast_IP("2607:f8b0:4005:802::200e"))
        self.assertTrue(kclxnet.is_global_unicast_IP("64:ff9b::800:1"))
        self.assertTrue(kclxnet.is_global_unicast_IP("220.181.108.89"))
    def test_is_multicast_IP(self):
        self.assertTrue(kclxnet.is_multicast_IP("239.255.255.255"))
    def test_is_loopback_IP(self):
        self.assertTrue(kclxnet.is_loopback_IP("127.0.0.1"))
    def test_is_link_local_unicast_IP(self):
        self.assertTrue(kclxnet.is_link_local_unicast_IP("fe80::2012:1"))
    def test_is_unspecified_IP(self):
        self.assertTrue(kclxnet.is_unspecified_IP("0.0.0.0"))

if __name__ == "__main__":
    unittest.main()
