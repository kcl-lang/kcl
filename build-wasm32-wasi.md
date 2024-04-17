# Build wasm32-wasi target

- Apple M1 should set `AR` and `CC` env
  - `export AR=/opt/homebrew/opt/llvm/bin/llvm-ar`
  - `export CC=/opt/homebrew/opt/llvm/bin/clang`
  - see https://github.com/surrealdb/surrealdb.wasm/issues/41
- `fslock` donot support wasm
  - https://github.com/brunoczim/fslock/issues/9
  - https://users.rust-lang.org/t/compile-bug-unresolved-import-crate-sys/70719
- Build wasm-wasi target
  - `cargo build --target=wasm32-wasi --release`

## build status

- compiler_base ok
- kclvm/macros ok
- kclvm/runtime ok
- kclvm/utils ok
- kclvm/version ok
- kclvm/span ok
- kclvm/error ok
- kclvm/ast ok
- kclvm/lexer ok
- kclvm/ast_pretty ok
- kclvm/config ok
- kclvm/parser ok
- kclvm/sema ok
- kclvm/query ok
- kclvm/loader ok
- kclvm/evaluator ok
- kclvm/compiler ok
- kclvm/driver ok
- kclvm/tools failed
- kclvm/runner failed
- kclvm/cmd failed
- kclvm/api failed

