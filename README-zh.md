<h1 align="center">KCL: Constraint-based Record & Functional Language</h1>

<p align="center">
<a href="./README.md">English</a> | <a href="./README-zh.md">简体中文</a>
</p>
<p align="center">
<a href="#介绍">介绍</a> | <a href="#特性">特性</a> | <a href="#场景">场景</a> | <a href="#安装">安装</a> | <a href="#快速开始">快速开始</a> | <a href="#文档">文档</a> | <a href="#贡献">贡献</a> | <a href="#路线规划">路线规划</a>
</p>

<p align="center">
  <img src="https://github.com/KusionStack/KCLVM/workflows/KCL/badge.svg">
  <img src="https://img.shields.io/badge/PRs-welcome-brightgreen.svg?style=flat-square">
  <img src="https://coveralls.io/repos/github/KusionStack/KCLVM/badge.svg">
  <img src="https://img.shields.io/github/release/KusionStack/KCLVM.svg">
  <img src="https://img.shields.io/github/license/KusionStack/KCLVM.svg">
</p>


## 介绍

Kusion 配置语言（KCL）是一个开源的基于约束的记录及函数语言。KCL 通过成熟的编程语言技术和实践来改进对大量繁杂配置的编写，致力于构建围绕配置的更好的模块化、扩展性和稳定性，更简单的逻辑编写，以及更快的自动化集成和良好的生态延展性。


## 特性

+ **简单易用**：源于 Python、Golang 等高级语言，采纳函数式编程语言特性，低副作用
+ **设计良好**：独立的 Spec 驱动的语法、语义、运行时和系统库设计
+ **快速建模**：以 [Schema](https://kusionstack.io/docs/reference/lang/lang/tour#schema) 为中心的配置类型及模块化抽象
+ **功能完备**：基于 [Config](https://kusionstack.io/docs/reference/lang/lang/codelab/simple)、[Schema](https://kusionstack.io/docs/reference/lang/lang/tour/#schema)、[Lambda](https://kusionstack.io/docs/reference/lang/lang/tour/#function)、[Rule](https://kusionstack.io/docs/reference/lang/lang/tour/#rule) 的配置及其模型、逻辑和策略编写
+ **可靠稳定**：依赖[静态类型系统](https://kusionstack.io/docs/reference/lang/lang/tour/#type-system)、[约束](https://kusionstack.io/docs/reference/lang/lang/tour/#validation)和[自定义规则](https://kusionstack.io/docs/reference/lang/lang/tour#rule)的配置稳定性
+ **强可扩展**：通过独立配置块[自动合并机制](https://kusionstack.io/docs/reference/lang/lang/tour/#-operators-1)保证配置编写的高可扩展性
+ **易自动化**：[CRUD APIs](https://kusionstack.io/docs/reference/lang/lang/tour/#kcl-cli-variable-override)，[多语言 SDK](https://kusionstack.io/docs/reference/lang/xlang-api/overview)，[语言插件](https://github.com/KusionStack/kcl-plugin) 构成的梯度自动化方案
+ **极致性能**：使用 Rust & C，[LLVM](https://llvm.org/) 实现，支持编译到本地代码和 [WASM](https://webassembly.org/) 的高性能编译时和运行时
+ **API 亲和**：原生支持 [OpenAPI](https://github.com/KusionStack/kcl-openapi)、 Kubernetes CRD， Kubernetes YAML 等 API 生态规范
+ **开发友好**：[语言工具](https://kusionstack.io/docs/reference/cli/kcl/) (Format，Lint，Test，Vet，Doc 等)、 [IDE 插件](https://github.com/KusionStack/vscode-kcl) 构建良好的研发体验
+ **安全可控**：面向领域，不原生提供线程、IO 等系统级功能，低噪音，低安全风险，易维护，易治理
+ **生产可用**：广泛应用在蚂蚁集团平台工程及自动化的生产环境实践中


## 场景

您可以将 KCL 用于

+ 生成静态配置数据如 JSON, YAML 等
+ 使用 schema 对配置数据进行建模并减少配置数据中的样板文件
+ 为配置数据定义带有规则约束的 schema 并对数据进行自动验证
+ 无副作用地组织、简化、统一和管理庞大的配置
+ 通过分块编写配置数据可扩展地管理庞大的配置
+ 与 [Kusion Stack](https://kusionstack.io) 一起，用作平台工程语言来交付现代应用程序


## 如何选择

简单的答案：

+ 如果你需要编写结构化的静态的 K-V，或使用 Kubernetes 原生的技术工具，建议选择 YAML
+ 如果你希望引入编程语言便利性以消除文本(如 YAML、JSON) 模板，有良好的可读性，或者你已是 Terraform 的用户，建议选择 HCL
+ 如果你希望引入类型功能提升稳定性，维护可扩展的配置文件，建议选择 CUE
+ 如果你希望以现代语言方式编写复杂类型和建模，维护可扩展的配置文件，原生的纯函数和策略，和生产级的性能和自动化，建议选择 KCL

稍后我们将提供更详细的功能和场景对比。


## 安装

从 Github releases 页面[下载](https://github.com/KusionStack/KCLVM/releases)，并且将 `{install-location}/kclvm/bin` 添加到您的环境变量中


## 快速开始

`./samples/fib.k` 是一个计算斐波那契数列的例子

```kcl
schema Fib:
    n1 = n - 1
    n2 = n1 - 1
    n: int
    value: int

    if n <= 1:
        value = 1
    elif n == 2:
        value = 1
    else:
        value = Fib {n = n1}.value + Fib {n = n2}.value

fib8 = Fib {n = 8}.value
```

我们可以通过执行如下命令得到 YAML 输出

```
kcl ./samples/fib.k
```

YAML 输出

```yaml
fib8: 21
```


## 文档

更多文档请访问[语言手册](https://kusionstack.io/docs/reference/lang/lang/tour)


## 贡献

参考[开发手册](./docs/dev_guide/1.about_this_guide.md).


## 路线规划

参考[KCLVM 路线规划](https://kusionstack.io/docs/governance/intro/roadmap/)


## 开源社区

欢迎访问 [KusionStack 社区](https://github.com/KusionStack/community) 加入我们。
