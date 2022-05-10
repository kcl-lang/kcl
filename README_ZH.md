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

+ [网站](https://kusionstack.io)

## 开发 & 贡献

参考[开发手册](./CONTRIBUTING.md).

## 许可

Apache License Version 2.0
