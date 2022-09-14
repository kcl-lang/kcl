<h1 align="center">KCL: Constraint-based Record & Functional Language</h1>

<p align="center">
<a href="./README.md">English</a> | <a href="./README-zh.md">简体中文</a>
</p>
<p align="center">
<a href="#introduction">Introduction</a> | <a href="#features">Features</a> | <a href="#what-is-it-for">What is it for</a> | <a href="#installation">Installation</a> | <a href="#showcase">Showcase</a> | <a href="#documentation">Documentation</a> | <a href="#contributing">Contributing</a> | <a href="#roadmap">Roadmap</a>
</p>

<p align="center">
  <img src="https://github.com/KusionStack/KCLVM/workflows/KCL/badge.svg">
  <img src="https://img.shields.io/badge/PRs-welcome-brightgreen.svg?style=flat-square">
  <img src="https://coveralls.io/repos/github/KusionStack/KCLVM/badge.svg">
  <img src="https://img.shields.io/github/release/KusionStack/KCLVM.svg">
  <img src="https://img.shields.io/github/license/KusionStack/KCLVM.svg">
</p>


## Introduction

Kusion Configuration Language (KCL) is an open source constraint-based record and functional language. KCL improves the writing of a large number of complex configurations through mature programming language technology and practice, and is committed to building better modularity, scalability and stability around configuration, simpler logic writing, fast automation and good ecological extensionality.


## Features

+ **Easy-to-use**: Originated from high-level languages ​​such as Python and Golang, incorporating functional language features with low side effects.
+ **Well-designed**: Independent Spec-driven syntax, semantics, runtime and system modules design.
+ **Quick modeling**: [Schema](https://kusionstack.io/docs/reference/lang/lang/tour#schema)-centric configuration types and modular abstraction.
+ **Rich capabilities**: Configuration with type, logic and policy based on [Config](https://kusionstack.io/docs/reference/lang/lang/codelab/simple), [Schema](https://kusionstack.io/docs/reference/lang/lang/tour/#schema), [Lambda](https://kusionstack.io/docs/reference/lang/lang/tour/#function), [Rule](https://kusionstack.io/docs/reference/lang/lang/tour/#rule).
+ **Stability**: Configuration stability built on [static type system](https://kusionstack.io/docs/reference/lang/lang/tour/#type-system), [constraints](https://kusionstack.io/docs/reference/lang/lang/tour/#validation), and [rules](https://kusionstack.io/docs/reference/lang/lang/tour#rule).
+ **Scalability**: High scalability through [automatic merge mechanism](https://kusionstack.io/docs/reference/lang/lang/tour/#-operators-1) of isolated config blocks.
+ **Fast automation**: Gradient automation scheme of [CRUD APIs](https://kusionstack.io/docs/reference/lang/lang/tour/#kcl-cli-variable-override), [multilingual SDKs](https://kusionstack.io/docs/reference/lang/xlang-api/overview), [language plugin](https://github.com/KusionStack/kcl-plugin)
+ **High performance**: High compile time and runtime performance using Rust & C and [LLVM](https://llvm.org/), and support compilation to native code and [WASM](https://webassembly.org/).
+ **API affinity**: Native support API ecological specifications such as [OpenAPI](https://github.com/KusionStack/kcl-openapi), Kubernetes CRD, Kubernetes YAML spec.
+ **Development friendly**: Friendly development experiences with rich [language tools](https://kusionstack.io/docs/reference/cli/kcl/) (Format, Lint, Test, Vet, Doc, etc.) and [IDE plugins](https://github.com/KusionStack/vscode-kcl).
+ **Safety & maintainable**: Domain-oriented, no system-level functions such as native threads and IO, low noise and security risk, easy maintenance and governance.
+ **Production-ready**: Widely used in production practice of platform engineering and automation at Ant Group.


## What is it for?

You can use KCL to

+ Generate low-level static configuration data like JSON, YAML, etc.
+ Reduce boilerplate in configuration data with the schema modeling.
+ Define schemas with rule constraints for configuration data and validate them automatically.
+ Organize, simplify, unify and manage large configurations without side effects.
+ Manage large configurations scalably with isolated configuration blocks.
+ Used as a platform engineering lang to deliver modern app with [Kusion Stack](https://kusionstack.io).


## How to choose?

The simple answer:

+ YAML is recommended if you need to write structured static K-V, or use Kubernetes' native tools
+ HCL is recommended if you want to use programming language convenience to remove boilerplate with good human readability, or if you are already a Terraform user
+ CUE is recommended if you want to use type system to improve stability and maintain scalable configurations
+ KCL is recommended if you want types and modeling like a modern language, scalable configurations, in-house pure functions and rules, and production-ready performance and automation

A detailed feature and scenario comparison will be coming later.


## Installation

[Download](https://github.com/KusionStack/KCLVM/releases) the latest release from GitHub and add `{install-location}/kclvm/bin` to the environment `PATH`.


## Showcase

`./samples/fib.k` is an example of calculating the Fibonacci sequence.

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

We can execute the following command to get a YAML output.

```
kcl ./samples/fib.k
```

YAML output

```yaml
fib8: 21
```


## Documentation

Detailed documentation is available at [KCL tour](https://kusionstack.io/docs/reference/lang/lang/tour)


## Contributing

See [Developing Guide](./docs/dev_guide/1.about_this_guide.md).


## Roadmap

See [KCLVM Roadmap](https://kusionstack.io/docs/governance/intro/roadmap/).
