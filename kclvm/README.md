# KCLVM

A high-performance implementation of KCL written in Rust that uses LLVM as the compiler backend.

## Building and Testing

Firstly, see [KCLVM CONTRIBUTING](../CONTRIBUTING.md) to build KCLVM. Secondly, we need to download the [Rust](https://www.rust-lang.org/), [SWIG](http://www.swig.org/), [LLVM 12](https://releases.llvm.org/download.html), and add the LLVM installation location to `LLVM_SYS_120_PREFIX` and the `$PATH`.

```shell
export LLVM_SYS_120_PREFIX=<your LLVM 12 install location>
export PATH=<your LLVM 12 install location>/bin:$PATH
```

To build everything, run:

```shell
make
```

After building, we can add the following command line parameters to use the KCL high-performance version:

```shell
kclvm_cli run main.k
```

To test, run:

```shell
make test
```

## Building and Testing in Docker

1. `make -C .. sh-in-docker`
2. `make build`
3. `export PATH=$PATH:/root/kclvm/_build/dist/ubuntu/kclvm/bin`
4. `kcl ./samples/hello.k`
5. `cd kclvm && make test`

## IDE

You can choose any IDE you like for development, but we recommend a combination of [VS Code](https://code.visualstudio.com/) and the [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=matklad.rust-analyzer) plugin.

## Notes

1. If you wanna start over, you `MUST` clean up all cached building files, such as `LLVM build files`, `kclvm/target`, etc.
2. If your updating-cargo-index is extremely slow, setup `~/.cargo/config` file.

```toml
[source.crates-io]
registry = "https://github.com/rust-lang/crates.io-index"
replace-with = 'ustc'

[source.ustc]
registry = "git://mirrors.ustc.edu.cn/crates.io-index"
```
