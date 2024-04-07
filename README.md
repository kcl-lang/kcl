<h1 align="center">KCL: Constraint-based Record & Functional Language</h1>

<p align="center">
<a href="./README.md">English</a> | <a href="./README-zh.md">简体中文</a>
</p>
<p align="center">
<a href="#introduction">Introduction</a> | <a href="#features">Features</a> | <a href="#what-is-it-for">What is it for</a> | <a href="#installation">Installation</a> | <a href="#documentation">Documentation</a> | <a href="#contributing">Contributing</a> | <a href="#roadmap">Roadmap</a>
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

## Introduction

KCL is an open-source, constraint-based record and functional language that enhances the writing of complex configurations, including those for cloud-native scenarios. With its advanced programming language technology and practices, KCL is dedicated to promoting better modularity, scalability, and stability for configurations. It enables simpler logic writing and offers ease of automation APIs and integration with homegrown systems.

<p align="center">
  <img src="https://kcl-lang.io/img/registry-and-ide.png">
</p>

## What is it for?

You can use KCL to

+ [Generate low-level static configuration data](https://kcl-lang.io/docs/user_docs/guides/configuration) such as JSON, YAML, etc., or [integrate with existing data](https://kcl-lang.io/docs/user_docs/guides/data-integration).
+ Reduce boilerplate in configuration data with the [schema modeling](https://kcl-lang.io/docs/user_docs/guides/schema-definition).
+ Define schemas with [rule constraints for configuration data and validate](https://kcl-lang.io/docs/user_docs/guides/validation) them automatically.
+ Organize, simplify, unify and manage large configurations without side effects through [gradient automation schemes and GitOps](https://kcl-lang.io/docs/user_docs/guides/automation).
+ Manage large configurations in a scalable way for different environments with [isolated configuration blocks](https://kcl-lang.io/docs/reference/lang/tour#config-operations).
+ Mutating or validating Kubernetes resources with [cloud-native configuration tool plugins](https://kcl-lang.io/docs/user_docs/guides/working-with-k8s/).
+ Used as a platform engineering programming language to deliver modern applications with [KusionStack](https://kusionstack.io).

## Features

+ **Easy-to-use**: Originated from high-level languages ​​such as Python and Golang, incorporating functional language features with low side effects.
+ **Well-designed**: Independent spec-driven syntax, semantics, runtime and system modules design.
+ **Quick modeling**: [Out-of-the-box modules](https://artifacthub.io/packages/search?org=kcl&sort=relevance&page=1) and [Schema](https://kcl-lang.io/docs/reference/lang/tour#schema)-centric configuration types and modular abstraction.
+ **Rich capabilities**: Configuration with type, logic and policy based on [Config](https://kcl-lang.io/docs/reference/lang/tour#config-operations), [Schema](https://kcl-lang.io/docs/reference/lang/tour#schema), [Lambda](https://kcl-lang.io/docs/reference/lang/tour#function), [Rule](https://kcl-lang.io/docs/reference/lang/tour#rule).
+ **Stability**: Configuration stability is achieved through a [static type system](https://kcl-lang.io/docs/reference/lang/tour/#type-system), [constraints](https://kcl-lang.io/docs/reference/lang/tour/#validation), and [rules](https://kcl-lang.io/docs/reference/lang/tour#rule).
+ **Scalability**: High scalability is assured with an [automatic merge mechanism](https://kcl-lang.io/docs/reference/lang/tour/#-operators-1) of isolated config blocks.
+ **Fast automation**: Gradient automation scheme of [CRUD APIs](https://kcl-lang.io/docs/reference/lang/tour/#kcl-cli-variable-override), [multilingual SDKs](https://kcl-lang.io/docs/reference/xlang-api/overview), and [language plugin](https://github.com/kcl-lang/kcl-plugin)
+ **High performance**: High compile-time and runtime performance using Rust & C and [LLVM](https://llvm.org/), and support compilation to native code and [WASM](https://webassembly.org/).
+ **API affinity**: Native support for ecological API specifications such as [OpenAPI](https://github.com/kcl-lang/kcl-openapi), Kubernetes CRD, Kubernetes Resource Model (KRM) spec.
+ **Developer-friendly**: Friendly development experiences with rich [language tools](https://kcl-lang.io/docs/tools/cli/kcl/) (Format, Lint, Test, Vet, Doc, package management tools etc.), and multiple [IDE extensions](https://kcl-lang.io/docs/tools/Ide/).
+ **Safety & maintainable**: Domain-oriented, no system-level functions such as native threads and IO, low noise and security risk, easy maintenance and governance.
+ **Rich multi-language SDK**: [Go](https://github.com/kcl-lang/kcl-go), [Python](https://github.com/kcl-lang/kcl-py), and [Java](https://github.com/kcl-lang/kcl-java) meet different scenarios and application use prelude.
+ **Integrations**: Abstract, mutate and validate manifests through [Kubectl KCL Plugin](https://github.com/kcl-lang/kubectl-kcl), [Kustomize KCL Plugin](https://github.com/kcl-lang/kustomize-kcl), [Helm KCL Plugin](https://github.com/kcl-lang/helm-kcl), [KPT KCL SDK](https://github.com/kcl-lang/kpt-kcl) or [Crossplane KCL Function](https://github.com/kcl-lang/crossplane-kcl).
+ **Production-ready**: Widely used in production practices of platform engineering and automation at Ant Group.

## How to choose?

A detailed feature and scenario comparison is [here](https://kcl-lang.io/docs/user_docs/getting-started/intro).

## Installation

For more information about installation, please check the [Installation Guide](https://kcl-lang.io/docs/user_docs/getting-started/install/) on the KCL official website.

## Documentation

Detailed documentation is available at [KCL Website](https://kcl-lang.io/)

## Contributing

See [Developing Guide](./docs/dev_guide/1.about_this_guide.md). You can also get started by opening the project in GitHub Codespaces.

[![Open in GitHub Codespaces](https://github.com/codespaces/badge.svg)](https://codespaces.new/kcl-lang/kcl)

## Roadmap

See [KCL Roadmap](https://github.com/kcl-lang/kcl/issues/882).

## Community

See the [community](https://github.com/kcl-lang/community) for ways to join us.

## License

[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2Fkcl-lang%2Fkcl.svg?type=large)](https://app.fossa.com/projects/git%2Bgithub.com%2Fkcl-lang%2Fkcl?ref=badge_large)
