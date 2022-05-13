# KCL 单元测试工具

## 简介

KCL 支持通过内置的 `kcl-test` 命令行工具和 `testing` 包提供了简易的测试框架。每个目录下的全部 KCL 文件是一个测试整体，每个 `_test.k` 中 `Test` 开头的 schema 是一个测试案例。

## 使用方式

假设有 hello.k 文件，代码如下:

```python
schema Person:
    name: str = "kcl"
    age: int = 1

hello = Person {
    name = "hello kcl"
    age = 102
}
```

构造 hello_test.k 测试文件，内容如下：

```python
schema TestPerson:
    a = Person{}
    assert a.name == 'kcl'

schema TestPerson_age:
    a = Person{}
    assert a.age == 1

schema TestPerson_ok:
    a = Person{}
    assert a.name == "kcl"
    assert a.age == 1
```

然后再目录下执行 `kcl-test` 命令:

```
$ kcl-test
ok   /pkg/to/app [365.154142ms]
$ 
```

## 失败的测试

将 hello_test.k 测试代码修改如下，构造失败的测试：

```python
# Copyright 2021 The KCL Authors. All rights reserved.

import testing

schema TestPerson:
    a = Person{}
    assert a.name == 'kcl2'

schema TestPerson_age:
    a = Person{}
    assert a.age == 123

schema TestPerson_ok:
    a = Person{}
    assert a.name == "kcl2"
    assert a.age == 1
```

测试输出的错误如下：

```
$ kcl-test
FAIL /pkg/to/app [354.153775ms]
---- <TestPerson> failed [48.817552ms]
     KCL Runtime Error: File /pkg/to/app/hello_test.k:7:
             assert a.name == 'kcl2'
     Assertion failure
---- <TestPerson_age> failed [47.515009ms]
     KCL Runtime Error: File /pkg/to/app/hello_test.k:11:
             assert a.age == 123
     Assertion failure
---- <TestPerson_ok> failed [47.26677ms]
     KCL Runtime Error: File /pkg/to/app/hello_test.k:15:
             assert a.name == "kcl2"
     Assertion failure
$
```

## 配置 option 参数

可以通过 testing 包指定面值类型的命令行参数：

```python
schema TestOptions:
    testing.arguments("name", "ktest")
    testing.arguments("age", "123")
    testing.arguments("int0", 0)
    testing.arguments("float0", 0.0)
    testing.arguments("bool-true", True)
    testing.arguments("bool-false", False)

    name = option("name")
    assert name == "ktest"

    age = option("age")
    assert age == 123

    assert option("int0") == 0
    assert option("float0") == 0.0
    assert option("bool-true") == True
    assert option("bool-false") == False
```

其中 `testing.arguments` 定义一组 key-value 参数，只有在当前的测试中有效。

option 参数也可以从 settings.yaml 文件读取。假设有 `./settings.yaml` 文件如下：

```yaml
  - key: app-name
    value: app
  - key: env-type
    value: prod
  - key: image
    value: reg.docker.inc.com/test-image
```

然后可以通过 `testing.setting_file("./settings.yaml")` 方式配置参数。同时依然支持 `testing.arguments()` 覆盖配置文件中的参数：

```py
schema TestOptions_setting:
    testing.setting_file("./settings.yaml")
    testing.arguments("file", "settings.yaml")

    assert option("app-name") == "app"
    assert option("file") == "settings.yaml"
```

testing.setting_file("settings.yaml") 则是从 yaml 文件加载对应的 key-value 参数。

## 测试插件

如果要测试的目录含有 `plugin.py` 和测试文件，自动切换到插件模式。那么将测试前设置 `KCL_PLUGINS_ROOT` 环境变量（不能再访问其他目录的插件）用于测试当前插件，测试完成之后恢复之前的 `KCL_PLUGINS_ROOT` 环境变量。

## 集成测试

目录含有 `*.k` 时自动执行集成测试，如果有 `stdout.golden` 则验证输出的结果，如果有 `stderr.golden` 则验证错误。支持 `settings.yaml` 文件定义命令行参数。

如果有 k 文件含有 `# kcl-test: ignore` 标注注释将忽略测试。

## 批量测试

- `kcl-test path` 执行指定目录的测试, 当前目录可以省略该参数
- `kcl-test -run=regexp` 可以执行匹配模式的测试
- `kcl-test ./...` 递归执行子目录的单元测试

## 命令行参数

```
$ kcl-test -h
NAME:
   kcl-go test - test packages

USAGE:
   kcl-go test [command options] [packages]

OPTIONS:
   --run value    Run only those tests matching the regular expression.
   --quiet, -q    Set quiet mode (default: false)
   --verbose, -v  Log all tests as they are run (default: false)
   --debug, -d    Run in debug mode (for developers only) (default: false)
   --help, -h     show help (default: false)
```
