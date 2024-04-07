<h1 align="center">KCL: 基于约束的记录及函数语言</h1>

<p align="center">
<a href="./README.md">English</a> | <a href="./README-zh.md">简体中文</a>
</p>
<p align="center">
<a href="#介绍">介绍</a> | <a href="#特性">特性</a> | <a href="#场景">场景</a> | <a href="#安装">安装</a> | <a href="#文档">文档</a> | <a href="#贡献">贡献</a> | <a href="#路线规划">路线规划</a>
</p>

<p align="center">
  <img src="https://github.com/kcl-lang/kcl/workflows/release/badge.svg">
  <img src="https://img.shields.io/badge/PRs-welcome-brightgreen.svg?style=flat-square">
  <img src="https://coveralls.io/repos/github/kcl-lang/kcl/badge.svg">
  <img src="https://img.shields.io/github/release/kcl-lang/kcl.svg">
  <img src="https://img.shields.io/github/license/kcl-lang/kcl.svg">
  <a href="https://www.bestpractices.dev/projects/7867"><img src="https://www.bestpractices.dev/projects/7867/badge"></a>
  <img src="https://img.shields.io/github/downloads/kcl-lang/kcl/total?label=Github%20downloads&logo=github">
  <img src="https://app.fossa.com/api/projects/git%2Bgithub.com%2Fkcl-lang%2Fkcl.svg?type=shield">
</p>

## 介绍

KCL 是一个开源的基于约束的记录及函数语言并通过成熟的编程语言技术和实践来改进对大量繁杂配置比如云原生 Kubernetes 配置场景的编写，致力于构建围绕配置的更好的模块化、扩展性和稳定性，更简单的逻辑编写，以及更简单的自动化和生态工具集成。

<p align="center">
  <img src="https://kcl-lang.io/img/registry-and-ide.png">
</p>

## 场景

您可以将 KCL 用于

+ [生成静态配置数据](https://kcl-lang.io/docs/user_docs/guides/configuration)如 JSON, YAML 等，或者[与已有的数据进行集成](https://kcl-lang.io/docs/user_docs/guides/data-integration)
+ [使用 Schema 对配置数据进行抽象建模](https://kcl-lang.io/docs/user_docs/guides/schema-definition)并减少配置数据中的样板文件
+ [为配置数据定义带有规则约束](https://kcl-lang.io/docs/user_docs/guides/validation)的 Schema 并对数据进行自动验证
+ [通过梯度自动化方案和 GitOps](https://kcl-lang.io/docs/user_docs/guides/automation)无副作用地组织、简化、统一和管理庞大的配置
+ 通过[分块编写配置数据](https://kcl-lang.io/docs/reference/lang/tour#config-operations)为不同的环境可扩展地管理庞大的配置
+ 通过与[云原生配置工具](https://kcl-lang.io/docs/user_docs/guides/working-with-k8s/)集成直接编辑或校验存量 Kubernetes 资源
+ 与 [KusionStack](https://kusionstack.io) 一起，用作平台工程语言来交付现代应用程序

## 特性

+ **简单易用**：源于 Python、Golang 等高级语言，采纳函数式编程语言特性，低副作用
+ **设计良好**：独立的规范驱动的语法、语义、运行时和系统库设计
+ **快速建模**：[开箱即用的模型库](https://artifacthub.io/packages/search?org=kcl&sort=relevance&page=1)和以 [Schema](https://kcl-lang.io/docs/reference/lang/tour#schema) 为中心的配置类型及模块化抽象
+ **功能完备**：基于 [Config](https://kcl-lang.io/docs/reference/lang/tour#config-operations)、[Schema](https://kcl-lang.io/docs/reference/lang/tour#schema)、[Lambda](https://kcl-lang.io/docs/reference/lang/tour#function)、[Rule](https://kcl-lang.io/docs/reference/lang/tour#rule) 的配置及其模型、逻辑和策略编写
+ **可靠稳定**：依赖[静态类型系统](https://kcl-lang.io/docs/reference/lang/tour/#type-system)、[约束](https://kcl-lang.io/docs/reference/lang/tour/#validation)和[自定义规则](https://kcl-lang.io/docs/reference/lang/tour#rule)的配置稳定性
+ **强可扩展**：通过独立配置块[自动合并机制](https://kcl-lang.io/docs/reference/lang/tour/#-operators-1)保证配置编写的高可扩展性
+ **易自动化**：[CRUD APIs](https://kcl-lang.io/docs/reference/lang/tour/#kcl-cli-variable-override)，[多语言 SDK](https://kcl-lang.io/docs/reference/xlang-api/overview)，[语言插件](https://github.com/kcl-lang/kcl-plugin) 构成的梯度自动化方案
+ **极致性能**：使用 Rust & C，[LLVM](https://llvm.org/) 实现，支持编译到本地代码和 [WASM](https://webassembly.org/) 的高性能编译时和运行时
+ **API 亲和**：原生支持 [OpenAPI](https://github.com/kcl-lang/kcl-openapi)、 Kubernetes CRD， Kubernetes Resource Model (KRM) 等 API 生态规范
+ **开发友好**：[语言工具](https://kcl-lang.io/docs/tools/cli/kcl/) (Format，Lint，Test，Vet，Doc, 包管理工具等) 和 [IDE 插件](https://kcl-lang.io/docs/tools/Ide/) 构建良好的研发体验
+ **安全可控**：面向领域，不原生提供线程、IO 等系统级功能，低噪音，低安全风险，易维护，易治理
+ **多语言 SDK**：[Go](https://github.com/kcl-lang/kcl-go)，[Python](https://github.com/kcl-lang/kcl-py) 和 [Java](https://github.com/kcl-lang/kcl-java) SDK 满足不同场景和应用使用需求
+ **生态集成**：通过 [Kubectl KCL 插件](https://github.com/kcl-lang/kubectl-kcl)、[Kustomize KCL 插件](https://github.com/kcl-lang/kustomize-kcl)、[Helm KCL 插件](https://github.com/kcl-lang/helm-kcl) 、[KPT KCL SDK](https://github.com/kcl-lang/kpt-kcl) 或者 [Crossplane KCL 函数](https://github.com/kcl-lang/crossplane-kcl) 直接编辑、校验或者抽象资源

+ **生产可用**：广泛应用在蚂蚁集团平台工程及自动化的生产环境实践中

## 如何选择

详细的功能和场景对比参考[这里](https://kcl-lang.io/docs/user_docs/getting-started/intro)。

## 安装

有关安装的更多信息，请查看 KCL 官网的[安装指南](https://kcl-lang.io/docs/user_docs/getting-started/install/)

## 文档

更多文档请访问[KCL 网站](https://kcl-lang.io/)

## 贡献

参考[开发手册](./docs/dev_guide/1.about_this_guide.md)。您也可以直接在 GitHub Codespaces 中打开该项目开始贡献。

[![用 GitHub Codespaces 打开](https://github.com/codespaces/badge.svg)](https://codespaces.new/kcl-lang/kcl)

## 路线规划

参考[KCL 路线规划](https://github.com/kcl-lang/kcl/issues/882)

## 社区

欢迎访问 [社区](https://github.com/kcl-lang/community) 加入我们。

## License

[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2Fkcl-lang%2Fkcl.svg?type=large)](https://app.fossa.com/projects/git%2Bgithub.com%2Fkcl-lang%2Fkcl?ref=badge_large)
