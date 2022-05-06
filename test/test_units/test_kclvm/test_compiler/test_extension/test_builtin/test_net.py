# Copyright 2020 The KCL Authors. All rights reserved.

import unittest

import kclvm.compiler.extension.builtin.system_module.net as net


class TestNetSystemModule(unittest.TestCase):
    def test_host_port(self):

        self.assertEqual(
            net.KMANGLED_split_host_port("B-K0NZJGH6-0048.local:80"),
            ["B-K0NZJGH6-0048.local", "80"],
        )
        self.assertEqual(
            net.KMANGLED_join_host_port("B-K0NZJGH6-0048.local", 80),
            "B-K0NZJGH6-0048.local:80",
        )

    def test_is_ip(self):
        cases = [
            # False cases
            {"value": "192.168.0,1", "expected": False},
            {"value": "192.168.0.", "expected": False},
            {"value": "192.168.", "expected": False},
            {"value": "192.", "expected": False},
            {"value": "", "expected": False},
            {"value": "256.168.0,1", "expected": False},
            {"value": "255.255.0.-1", "expected": False},
            {"value": "192.0022.1.1", "expected": False},
            {"value": "492.10.123.12313", "expected": False},
            {"value": "0xFF.0xFF.0xFF.0xFF", "expected": False},
            {"value": "2001:0db8:3c4d:0015+0000:0000:1a2f:1a2b", "expected": False},
            # True cases
            {"value": "255.255.0.1", "expected": True},
            {"value": "1.0.0.0", "expected": True},
            {"value": "192.190.0.1", "expected": True},
            {"value": "128.0.0.0", "expected": True},
            {"value": "172.16.0.0", "expected": True},
            {"value": "172.31.255.255", "expected": True},
            {"value": "169.254.0.1", "expected": True},
            {"value": "191.255.255.255", "expected": True},
            {"value": "223.255.255.0", "expected": True},
            {"value": "2001:0db8:3c4d:0015:0000:0000:1a2f:1a2b", "expected": True},
        ]
        for case in cases:
            value, expected = case["value"], case["expected"]
            self.assertEqual(net.KMANGLED_is_IP(value), expected)

    def test_is_multicast_IP(self):
        cases = [
            {"value": "239.255.255.255", "expected": True},
        ]
        for case in cases:
            value, expected = case["value"], case["expected"]
            self.assertEqual(net.KMANGLED_is_multicast_IP(value), expected)

    def test_is_loopback_IP(self):
        cases = [
            {"value": "127.0.0.1", "expected": True},
        ]
        for case in cases:
            value, expected = case["value"], case["expected"]
            self.assertEqual(net.KMANGLED_is_loopback_IP(value), expected)

    def test_is_link_local_multicast_IP(self):
        cases = [
            {"value": "224.0.0.0", "expected": False},
        ]
        for case in cases:
            value, expected = case["value"], case["expected"]
            self.assertEqual(net.KMANGLED_is_link_local_multicast_IP(value), expected)

    def test_is_link_local_unicast_IP(self):
        cases = [
            {"value": "fe80::2012:1", "expected": True},
        ]
        for case in cases:
            value, expected = case["value"], case["expected"]
            self.assertEqual(net.KMANGLED_is_link_local_unicast_IP(value), expected)

    def test_is_global_unicast_IP(self):
        cases = [
            {"value": "220.181.108.89", "expected": True},
        ]
        for case in cases:
            value, expected = case["value"], case["expected"]
            self.assertEqual(net.KMANGLED_is_global_unicast_IP(value), expected)

    def test_is_unspecified_IP(self):
        cases = [
            {"value": "0.0.0.0", "expected": True},
        ]
        for case in cases:
            value, expected = case["value"], case["expected"]
            self.assertEqual(net.KMANGLED_is_unspecified_IP(value), expected)


if __name__ == "__main__":
    unittest.main(verbosity=2)
