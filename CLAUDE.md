# KCL Repository Overview for AI Assistants

This document provides a comprehensive overview of the KCL repository to help AI assistants understand the codebase for future work.

## What is KCL?

**KCL (KCL Constraint-based Record & Functional Language)** is an open-source configuration and policy language designed for cloud-native scenarios. It's a CNCF Sandbox project that enhances the writing of complex configurations through advanced programming language technology.

### Main Purpose
- Generate low-level static configuration data (JSON, YAML) for cloud-native applications
- Reduce boilerplate in configuration through schema modeling
- Define and validate configuration data with rule constraints
- Manage large-scale configurations with GitOps and automation
- Mutate/validate Kubernetes resources through various tool plugins
- Platform engineering language for modern application delivery (used with KusionStack)

### Key Use Cases
- Kubernetes configuration management and abstraction
- Terraform resource model abstraction
- Configuration validation and constraint checking
- Large-scale infrastructure as code
- Platform engineering and GitOps workflows

### Production Users
Ant Group, Youzan, and Huawei are notable production users managing large-scale Kubernetes deployments.

## Repository Structure

### Top-Level Organization

```
/crates/              Core KCL VM and compiler implementation (main codebase)
/compiler_base/      Base compiler libraries and utilities (WIP, rustc-derived)
/cli/                Command-line interface binary wrapper
/test/               Integration and grammar tests
  /grammar/          Extensive grammar test cases
  /integration/      Integration test suites
/docs/               Developer guides and documentation
  /dev_guide/        Development guide (architecture, quick start, etc.)
  /design/           Design documents
/scripts/            Build and release automation scripts
/.github/workflows/  CI/CD pipelines for multiple platforms
```

### Key Files
- `Makefile` - Top-level build orchestration
- `run.sh` - Build and release automation script
- `VERSION` - Current version: **0.11.2**
- `LICENSE` - Apache License 2.0
- `README.md` - Project documentation

## Technology Stack

### Primary Language: Rust
- 362+ Rust source files
- ~32,673 lines of Rust code in core modules
- **Requires Rust 1.88+** for building
- Rust 2021 edition

### Secondary Languages
- **KCL** - The language itself (.k files for examples and tests)
- **Python** - Integration tests and test infrastructure
- **Shell/Bash** - Build scripts and automation
- **C/C++** - Runtime interop and FFI interfaces

### Key Dependencies
- **LLVM 12** - Compiler backend (optional, for high-performance compilation)
- **Protobuf** - API definitions and RPC communication
- **WASM** - WebAssembly compilation target support
- **tokio** - Async runtime (for LSP and server)
- **salsa** - Incremental computation (for LSP)

## Architecture

### Compilation Pipeline

```
Source Code (.k files)
    ↓
[Lexer] → Tokens
    ↓
[Parser] → AST
    ↓
[Resolver/Sema] → Semantic Analysis & Type Checking
    ↓
[Compiler] → IR (LLVM IR or AST-based)
    ↓
[Evaluator/Runner] → Execution
    ↓
Output (YAML/JSON)
```

### Key Components (/crates)

**Frontend (Parsing & Analysis):**
- `kclvm-lexer` - Lexical analysis and tokenization
- `kclvm-parser` - Parse KCL source into AST
- `kclvm-ast` - Abstract Syntax Tree definitions and walker
- `kclvm-ast-pretty` - AST formatting and pretty-printing
- `kclvm-span` - Source code span/position tracking
- `kclvm-error` - Error handling and diagnostics

**Semantic Analysis:**
- `kclvm-sema` - Semantic analysis, type checking, and validation
- `kclvm-loader` - Module loading and dependency management
- `kclvm-query` - Code query and information retrieval

**Compilation & Execution:**
- `kclvm-compiler` - Main compilation logic with optional LLVM backend
- `kclvm-evaluator` - Expression evaluation engine
- `kclvm-runner` - Program execution environment
- `kclvm-driver` - Compilation driver and orchestration

**Runtime:**
- `kclvm-runtime` - Runtime support libraries with extensive standard library
  - Value representation and type system
  - Standard library modules: json, yaml, base64, regex, crypto, datetime, math, net, etc.
  - Template rendering (handlebars)
  - File I/O and manifests

**Tooling:**
- `kclvm-tools` - Development tools
  - Format, Lint, Fix, Vet
  - Testing infrastructure
  - **LSP** (Language Server) - Full IDE support with autocomplete, goto-definition, diagnostics
- `kclvm-api` - Public API layer for multi-language SDKs
- `kclvm-cmd` - CLI command implementation

**Utilities:**
- `kclvm-config` - Configuration parsing
- `kclvm-version` - Version management
- `kclvm-utils` - Common utilities
- `kclvm-primitives` - Primitive type definitions
- `kclvm-macros` - Procedural macros

### Language Server Architecture
- Salsa-based incremental compilation for performance
- VFS (Virtual File System) for handling unsaved changes
- Thread pool for concurrent request handling
- Event-driven architecture (Tasks + LSP Messages)
- Compile unit discovery for projects without explicit config
- Located at: `/crates/tools/src/LSP`

## Build System

### Build Tools
- **Cargo** - Primary Rust build system (workspace-based with 20+ crates)
- **Make** - Top-level orchestration
- **Docker** - Containerized build environment (recommended: `kcllang/kcl-builder`)

### Common Build Commands

```bash
make build          # Standard build
make release        # Release build
make check          # Type checking
make build-lsp      # Language server
make build-wasm     # WASM target
```

### Build Features
- Workspace with 20+ member crates
- Optional `llvm` feature flag for high-performance backend
- Support for multiple targets: native, WASM (wasm32-wasip1), WASM-unknown
- Cross-platform: Linux (AMD64, ARM64), macOS (AMD64, ARM64), Windows (MinGW)
- Release profile optimized for size (opt-level = "z", LTO enabled)

### Major Dependencies
- **inkwell** - LLVM bindings (optional)
- **serde/serde_json** - Serialization
- **serde_yaml_ng** - YAML support (note: migrated from serde_yaml)
- **prost/protobuf** - Protocol buffers
- **tokio** - Async runtime
- **lsp-server/lsp-types** - Language Server Protocol
- **salsa** - Incremental computation
- **rustc_lexer** - Rust's lexer for tokenization
- **petgraph** - Graph data structures
- **regex/fancy-regex** - Regular expressions

## Testing

### Testing Strategy

**1. Unit Tests:**
- Cargo-based unit tests across all crates
- Command: `make test` or `cargo test --workspace`
- Code coverage via `cargo llvm-cov`

**2. Grammar Tests:**
- Extensive grammar test suite in `/test/grammar`
- Python-based pytest framework
- Parallel execution: `pytest -v -n 5`
- Tests both AST evaluator and fast evaluator
- Command: `make test-grammar`

**3. Integration Tests:**
- `tests/grammar` - Python integration tests

**4. Runtime Tests:**
- Python-based runtime library tests
- Command: `make test-runtime`

### CI/CD
Comprehensive GitHub Actions workflows (11 pipelines) for:
- Linux AMD64 and ARM64
- macOS AMD64 and ARM64
- Windows (MSVC and MinGW)
- Alpine Linux (musl)
- CentOS 7
- WASM/WASI
- Compiler base tests

## Documentation

### Primary Documentation Sources

1. **Official Website:** https://kcl-lang.io/
   - User guides, language tour, API reference
   - Installation instructions
   - Tutorial and examples

2. **Repository Documentation:**
   - `/README.md` - Project overview and quick start
   - `/README-zh.md` - Chinese documentation
   - `/docs/dev_guide/` - Developer guide
     - `1.about_this_guide.md` - Guide overview
     - `2.quick_start.md` - Building and setup
     - `3.coding_conventions.md` - Code style
     - `4.architecture.md` - Compiler architecture
     - `5.source_code.md` - Source code structure
     - `6.languager_server.md` - LSP implementation

3. **Governance:**
   - `/GOVERNANCE.md` - Project governance (CNCF sandbox)
   - `/CODE_OF_CONDUCT.md` - Community guidelines
   - `/ADOPTERS.md` - Production users
   - `/MAINTAINERS` - Maintainer list

## Language Design Principles

1. **Spec-driven**: Independent syntax and semantics specification
2. **Functional**: Low side-effects, no system-level operations (no threads/IO)
3. **Constraint-based**: Schema + Rule + Lambda for configuration validation
4. **High Performance**: Rust + LLVM compilation, WASM support
5. **API-first**: Multi-language SDKs (Rust, Go, Python, .NET, Java, Node.js)
6. **Cloud-native**: Native support for OpenAPI, K8s CRD, KRM spec
7. **Type Safety**: Static type system with constraints and validation rules

## Development Workflow

### Setting Up Development Environment

```bash
# Recommended: Use Docker builder image
docker pull kcllang/kcl-builder

# Or install dependencies locally
# - Rust 1.88+
# - LLVM 12 (optional, for high-performance backend)
# - Python 3.x (for tests)
# - Protobuf compiler

# Build the project
make build

# Run tests
make test
make test-grammar
```

### IDE Support
- Full Language Server Protocol (LSP) support
- VSCode extension available
- Rust-analyzer recommended for Rust development
- Dev container configuration in `.devcontainer/`

## Notable Features

1. **Production-Ready:** Used by major companies for large-scale Kubernetes management
2. **Multi-Platform:** Exceptional cross-platform support including ARM and WASM
3. **Rich Ecosystem:**
   - Kubectl, Kustomize, Helm, KPT, Crossplane plugins
   - Package registry at artifacthub.io
   - Integration with GitOps workflows
4. **Developer Experience:**
   - Full LSP implementation with IDE support
   - Comprehensive tooling (format, lint, test, vet)
   - Extensive test coverage and fuzzing
5. **Performance Focus:**
   - Optional LLVM backend for native code compilation
   - WASM compilation target
   - Size-optimized release builds

## Git Workflow

- **Main branch:** `main` (use this for PRs)
- **License:** Apache License 2.0
- **Current version:** 0.11.2
- **Recent focus areas:** YAML serialization fixes, LSP formatter improvements, documentation

## Important Notes for AI Assistants

1. **Always check existing code patterns** - This is a mature codebase with established conventions
2. **Test coverage is critical** - Add tests for any new functionality
3. **Performance matters** - This is used in production for large-scale configurations
4. **Cross-platform support** - Consider multiple platforms when making changes
5. **Documentation** - Keep docs in sync with code changes
6. **The codebase uses workspaces** - Changes may affect multiple crates
7. **LLVM backend is optional** - Code should work with or without it
8. **Recent migration from serde_yaml to serde_yaml_ng** - Use the new library

## Quick Reference

| Task | Command |
|------|---------|
| Build | `make build` |
| Test | `make test` |
| Format | `cargo fmt` |
| Lint | `cargo clippy` |
| Grammar tests | `make test-grammar` |
| Build LSP | `make build-lsp` |
| Build WASM | `make build-wasm` |
| Release build | `make release` |

## Support

- **Official website:** https://kcl-lang.io/
- **GitHub:** Current repository
- **CNCF:** Sandbox project with formal governance
