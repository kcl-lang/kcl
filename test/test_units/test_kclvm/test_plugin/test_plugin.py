# Copyright 2020 The KCL Authors. All rights reserved.

import kclvm.compiler.extension.plugin as plugin

hello_plugin = plugin.get_plugin("kcl_plugin.hello")


def test_reset_plugin():
    plugin.reset_plugin(plugin.get_plugin_root())


def test_plugin():
    assert "hello" in plugin.get_plugin_names()


def test_plugin_info():
    info = plugin.get_info("hello")
    assert info["name"] == "hello"


def test_plugin_hello_add():
    assert hello_plugin.add(1, 2) == 3


def test_plugin_hello_tolower():
    assert hello_plugin.tolower("KCL") == "kcl"


def test_plugin_hello_update_dict():
    assert hello_plugin.update_dict({"name": 123}, "name", "kcl")["name"] == "kcl"


def test_plugin_hello_list_append():
    data = hello_plugin.list_append(["abc"], "name", 123)
    assert len(data) == 3
    assert data[0] == "abc"
    assert data[1] == "name"
    assert data[2] == 123


def test_plugin_hello_foo():
    v = hello_plugin.foo("aaa", "bbb", x=123, y=234, abcd=1234)
    assert len(v) == 5
    assert v["a"] == "aaa"
    assert v["b"] == "bbb"
    assert v["x"] == 123
    assert v["y"] == 234
    assert v["abcd"] == 1234

    v = hello_plugin.foo("aaa", "bbb", x=123)
    assert len(v) == 3
    assert v["a"] == "aaa"
    assert v["b"] == "bbb"
    assert v["x"] == 123
