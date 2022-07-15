# KCL

![license](https://img.shields.io/badge/license-Apache--2.0-green.svg)
[![Continuous Integration](https://github.com/KusionStack/KCLVM/actions/workflows/github-actions.yaml/badge.svg)](https://github.com/KusionStack/KCLVM/actions?query=branch%3Amain)

[English](./README.md)

Kusion 配置语言（KCL）是一种开源配置语言，主要用于 [Kusion Stack](https://kusionstack.io) 开放协同技术栈。并且 KCL 是一种基于声明性和面向对象编程 (OOP) 范式等概念，用于配置和策略场景的静态类型语言。

## 核心特性

+ **简单**
  + 源于 Python、Golang，融入函数语言特性
  + 吸收语句、表达式、条件、循环等语言元素
  + 类型和数据分离，Schema 声明配置定义
+ **稳定**
  + 强不可变约束
  + 编译时类型推导、类型检查
  + Rule 策略定义：以属性为中心的约束表达式、根据约束查询结果
  + 可测试：语言内置 assert 断言、print 打印和测试工具
+ **可扩展**
  + 配置合并：编译时配置依赖图代换
  + 配置属性运算符：满足配置覆盖、合并、添加和删除等需求
  + 配置复用：丰富的内置数据结构和语法语义，轻松扩展同一份配置到不同场景
+ **工程化**
  + Schema 单一继承和声明性模型复用和组装
  + 工具和API 粒度的配置自动化“增删改查”
  + 丰富的内置函数和系统库
  + 顶层数据动态导入
  + 代码组织：模块和包
  + [插件系统](https://github.com/KusionStack/kcl-plugin)：复用通用编程语言生态。
  + [OpenAPI 模型支持](https://github.com/KusionStack/kcl-openapi)：Swagger 与 Schema 双向转换，Kubernetes CRD 转换为 Schema
+ **高性能**
  + 配合 LLVM 优化器、支持编译到本地代码和 WASM 等格式并高效执行

## 安装 & 文档

### 如何安装

从 Github releases 页面[下载](https://github.com/KusionStack/KCLVM/releases)，并且将 `{install-location}/kclvm/bin` 添加到您的环境变量中

### 快速开始

`./samples/fib.k` 是一个计算斐波那契数列的例子

```kcl
schema Fib:
    n1: int = n - 1
    n2: int = n1 - 1
    n: int
    value: int

    if n <= 1:
        value = 1
    elif n == 2:
        value = 1
    else:
        value = Fib {n: n1}.value + Fib {n: n2}.value

fib8 = Fib {n: 8}.value
```

我们可以通过执行如下命令得到 YAML 输出

```
kcl ./samples/fib.k
```

YAML 输出

```yaml
fib8: 21
```

### 文档

更多文档请访问 https://kusionstack.io

## 开发 & 贡献

### 开发

参考[开发手册](./docs/dev_guide/1.about_this_guide.md).

### 路线规划

参考[KCLVM 路线规划](https://kusionstack.io/docs/governance/intro/roadmap#kclvm-%E8%B7%AF%E7%BA%BF%E8%A7%84%E5%88%92)

## 许可

Apache License Version 2.0
