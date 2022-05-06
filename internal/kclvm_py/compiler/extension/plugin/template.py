# Copyright 2020 The KCL Authors. All rights reserved.


def get_plugin_template_code(plugin_name: str) -> str:
    return f'''# Copyright 2020 The KCL Authors. All rights reserved.

INFO = {{
    'name': '{plugin_name}',
    'describe': '{plugin_name} doc',
    'long_describe': 'long describe',
    'version': '0.0.1',
}}


global_int: int = 0


def set_global_int(v: int):
    global global_int
    global_int = v


def get_global_int() -> int:
    return global_int


def say_hello(msg: str):
    print('{plugin_name}.say_hello:', msg)
    return None


def add(a: int, b: int) -> int:
    """add two numbers, and return result"""
    return a + b


def tolower(s: str) -> str:
    return s.lower()


def update_dict(d: dict, key: str, value: str) -> dict:
    d[key] = value
    return d


def list_append(l: list, *values) -> list:
    for v in values:
        l.append(v)
    return l


def foo(a, b, *, x, **values):
    print(a, b, x, values)
    return {{'a': a, 'b': b, 'x': x, **values}}
'''


def get_plugin_test_template_code(plugin_name: str) -> str:
    return """# Copyright 2020 The KCL Authors. All rights reserved.

# python3 -m pytest

import plugin


def test_add():
    assert plugin.add(1, 2) == 3


def test_tolower():
    assert plugin.tolower('KCL') == 'kcl'


def test_update_dict():
    assert plugin.update_dict({{'name': 123}}, 'name', 'kcl')['name'] == 'kcl'


def test_list_append():
    l = plugin.list_append(['abc'], 'name', 123)
    assert len(l) == 3
    assert l[0] == 'abc'
    assert l[1] == 'name'
    assert l[2] == 123


def test_foo():
    v = plugin.foo('aaa', 'bbb', x=123, y=234, abcd=1234)
    assert len(v) == 5
    assert v['a'] == 'aaa'
    assert v['b'] == 'bbb'
    assert v['x'] == 123
    assert v['y'] == 234
    assert v['abcd'] == 1234

    v = plugin.foo('aaa', 'bbb', x=123)
    assert len(v) == 3
    assert v['a'] == 'aaa'
    assert v['b'] == 'bbb'
    assert v['x'] == 123
"""
