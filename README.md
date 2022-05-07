# KCL

![license](https://img.shields.io/badge/license-Apache--2.0-green.svg)

[中文](./README_ZH.md)

Kusion Configuration Language (KCL) is an open source configuration language mainly used in [Kusion Stack](https://kusionstack.io). KCL is a statically typed language for configuration and policy scenarios, based on concepts such as declarative and Object-Oriented Programming (OOP) paradigms.

## Core Features

+ **Simple**
  + Originated from Python and Golang, incorporating functional language features.
  + Absorbs integrated language elements such as statements, expressions, conditions, loops, etc.
  + Type and data separation, schema declaration configuration definition.
+ **Stable**
  + Strong immutable constraint.
  + Compile-time type deduction, type checking.
  + Rule policy definition: attribute-centric constraint expressions, query results based on constraints.
  + Testable: assert, print, and test tools.
+ **Scalable**
  + Configuration unification: compile-time configuration dependency graph substitution.
  + Configuration attribute operators: meet the needs of configuration override, merge, add and delete, etc.
  + Configuration reuse: rich built-in data structures and syntax semantics, easily to expand one configuration of different scenarios.
+ **Engineering**
  + Schema single inheritance and declarative model reuse and assembly.
  + Tool & API granular configuration automation.
  + Rich built-in functions and system libraries.
  + Top-level dynamic data input.
  + Code organization: modules and packages.
  + [Plug-in system](https://github.com/KusionStack/kcl-plugin): reuse common programming language ecology.
  + [OpenAPI model support](https://github.com/KusionStack/kcl-openapi): Swagger and KCL schema bidirectional conversion, Kubernetes CRD conversion to KCL schema.
+ **High Performance**
  + Works with the LLVM optimizer, supports compilation to native code and formats like WASM and executes efficiently.

## Installing & Documentation

+ [Website](https://kusionstack.io)

## Developing & Contributing

See our [developing guide](./CONTRIBUTING.md).

## License

Apache License Version 2.0
