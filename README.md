# KCL

![license](https://img.shields.io/badge/license-Apache--2.0-green.svg)
[![Continuous Integration](https://github.com/KusionStack/KCLVM/actions/workflows/github-actions.yaml/badge.svg)](https://github.com/KusionStack/KCLVM/actions?query=branch%3Amain)

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

### How to install

[Download](https://github.com/KusionStack/KCLVM/releases) the latest release from GitHub and add `{install-location}/kclvm/bin` to environment PATH.

### Quick Showcase

`./samples/fib.k` is an example of calculating the Fibonacci sequence.

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

We can execute the following command to get a YAML output.

```
kcl ./samples/fib.k
```

YAML output

```yaml
fib8: 21
```

### Documentation

Detailed documentation is available at https://kusionstack.io

## Developing & Contributing

### Developing

See [Developing Guide](./docs/dev_guide/1.about_this_guide.md).

### Roadmap

See [KCLVM Roadmap](https://kusionstack.io/docs/governance/intro/roadmap#kclvm-%E8%B7%AF%E7%BA%BF%E8%A7%84%E5%88%92)

## License

Apache License Version 2.0
