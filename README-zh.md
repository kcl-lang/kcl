<h1 align="center">KCL: 基于约束的记录及函数语言</h1>

<p align="center">
<a href="./README.md">English</a> | <a href="./README-zh.md">简体中文</a>
</p>
<p align="center">
<a href="#介绍">介绍</a> | <a href="#特性">特性</a> | <a href="#场景">场景</a> | <a href="#安装">安装</a> | <a href="#快速开始">快速开始</a> | <a href="#文档">文档</a> | <a href="#贡献">贡献</a> | <a href="#路线规划">路线规划</a>
</p>

<p align="center">
  <img src="https://github.com/KusionStack/KCLVM/workflows/release/badge.svg">
  <img src="https://img.shields.io/badge/PRs-welcome-brightgreen.svg?style=flat-square">
  <img src="https://coveralls.io/repos/github/KusionStack/KCLVM/badge.svg">
  <img src="https://img.shields.io/github/release/KusionStack/KCLVM.svg">
  <img src="https://img.shields.io/github/license/KusionStack/KCLVM.svg">
</p>

## 介绍

Kusion 配置语言（KCL）是一个开源的基于约束的记录及函数语言。KCL 通过成熟的编程语言技术和实践来改进对大量繁杂配置比如云原生场景的编写，致力于构建围绕配置的更好的模块化、扩展性和稳定性，更简单的逻辑编写，以及更快的自动化集成和良好的生态延展性。

## 场景

您可以将 KCL 用于

+ [生成静态配置数据](https://kcl-lang.io/docs/user_docs/guides/configuration)如 JSON, YAML 等，或者[与已有的数据进行集成](https://kcl-lang.io/docs/user_docs/guides/data-integration)
+ [使用 schema 对配置数据进行建模](https://kcl-lang.io/docs/user_docs/guides/schema-definition)并减少配置数据中的样板文件
+ [为配置数据定义带有规则约束](https://kcl-lang.io/docs/user_docs/guides/validation)的 schema 并对数据进行自动验证
+ [通过梯度自动化方案](https://kcl-lang.io/docs/user_docs/guides/automation)无副作用地组织、简化、统一和管理庞大的配置
+ 通过[分块编写配置数据](https://kcl-lang.io/docs/reference/lang/tour#config-operations)可扩展地管理庞大的配置
+ 与 [Kusion Stack](https://kusionstack.io) 一起，用作平台工程语言来交付现代应用程序

## 特性

+ **简单易用**：源于 Python、Golang 等高级语言，采纳函数式编程语言特性，低副作用
+ **设计良好**：独立的 Spec 驱动的语法、语义、运行时和系统库设计
+ **快速建模**：以 [Schema](https://kcl-lang.io/docs/reference/lang/tour#schema) 为中心的配置类型及模块化抽象
+ **功能完备**：基于 [Config](https://kcl-lang.io/docs/reference/lang/tour#config-operations)、[Schema](https://kcl-lang.io/docs/reference/lang/tour#schema)、[Lambda](https://kcl-lang.io/docs/reference/lang/tour#function)、[Rule](https://kcl-lang.io/docs/reference/lang/tour#rule) 的配置及其模型、逻辑和策略编写
+ **可靠稳定**：依赖[静态类型系统](https://kcl-lang.io/docs/reference/lang/tour/#type-system)、[约束](https://kcl-lang.io/docs/reference/lang/tour/#validation)和[自定义规则](https://kcl-lang.io/docs/reference/lang/tour#rule)的配置稳定性
+ **强可扩展**：通过独立配置块[自动合并机制](https://kcl-lang.io/docs/reference/lang/tour/#-operators-1)保证配置编写的高可扩展性
+ **易自动化**：[CRUD APIs](https://kcl-lang.io/docs/reference/lang/tour/#kcl-cli-variable-override)，[多语言 SDK](https://kcl-lang.io/docs/reference/xlang-api/overview)，[语言插件](https://github.com/KusionStack/kcl-plugin) 构成的梯度自动化方案
+ **极致性能**：使用 Rust & C，[LLVM](https://llvm.org/) 实现，支持编译到本地代码和 [WASM](https://webassembly.org/) 的高性能编译时和运行时
+ **API 亲和**：原生支持 [OpenAPI](https://github.com/KusionStack/kcl-openapi)、 Kubernetes CRD， Kubernetes YAML 等 API 生态规范
+ **开发友好**：[语言工具](https://kcl-lang.io/docs/tools/cli/kcl/) (Format，Lint，Test，Vet，Doc 等)、 [IDE 插件](https://github.com/KusionStack/vscode-kcl) 构建良好的研发体验
+ **安全可控**：面向领域，不原生提供线程、IO 等系统级功能，低噪音，低安全风险，易维护，易治理
+ **生态集成**：通过 [Kustomize KCL 插件](https://github.com/KusionStack/kustomize-kcl), [Helm KCL 插件](https://github.com/KusionStack/helm-kcl) 或者 [KPT KCL SDK](https://github.com/KusionStack/kpt-kcl-sdk) 直接编辑或校验资源
+ **生产可用**：广泛应用在蚂蚁集团平台工程及自动化的生产环境实践中

## 如何选择

简单的答案：

+ 如果你需要编写结构化的静态的 K-V，或使用 Kubernetes 原生的技术工具，建议选择 YAML
+ 如果你希望引入编程语言便利性以消除文本(如 YAML、JSON) 模板，有良好的可读性，或者你已是 Terraform 的用户，建议选择 HCL
+ 如果你希望引入类型功能提升稳定性，维护可扩展的配置文件，建议选择 CUE
+ 如果你希望以现代语言方式编写复杂类型和建模，维护可扩展的配置文件，原生的纯函数和策略，和生产级的性能和自动化，建议选择 KCL

更详细的功能和场景对比参考[这里](https://kcl-lang.io/docs/user_docs/getting-started/intro)。

## 安装

从 Github releases 页面[下载](https://github.com/KusionStack/KCLVM/releases)，并且将 `{install-location}/kclvm/bin` 添加到您的环境变量中

## 快速开始

`./samples/kubernetes.k` 是一个生成 kubernetes 资源的例子

```python
apiVersion = "apps/v1"
kind = "Deployment"
metadata = {
    name = "nginx"
    labels.app = "nginx"
}
spec = {
    replicas = 3
    selector.matchLabels = metadata.labels
    template.metadata.labels = metadata.labels
    template.spec.containers = [
        {
            name = metadata.name
            image = "${metadata.name}:1.14.2"
            ports = [{ containerPort = 80 }]
        }
    ]
}
```

我们可以通过执行如下命令得到 YAML 输出

```bash
kcl ./samples/kubernetes.k
```

YAML 输出

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: nginx
  labels:
    app: nginx
spec:
  replicas: 3
  selector:
    matchLabels:
      app: nginx
  template:
    metadata:
      labels:
        app: nginx
    spec:
      containers:
      - name: nginx
        image: nginx:1.14.2
        ports:
        - containerPort: 80
```

## 文档

更多文档请访问[KCL 网站](https://kcl-lang.io/)

## 贡献

参考[开发手册](./docs/dev_guide/1.about_this_guide.md).

## 路线规划

参考[KCL 路线规划](https://github.com/KusionStack/KCLVM/issues/29)

## 开源社区

欢迎访问 [社区](https://github.com/KusionStack/community) 加入我们。
