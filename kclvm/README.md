# KCLVM

A high-performance implementation of KCL written in Rust that uses LLVM as the compiler backend.

## Building and Testing

Firstly, see [KCLVM CONTRIBUTING](../CONTRIBUTING.md) to build KCLVM. Secondly, we need to download the [Rust](https://www.rust-lang.org/), [SWIG](http://www.swig.org/), [LLVM 12](https://releases.llvm.org/download.html), and add the LLVM installation location to `LLVM_SYS_120_PREFIX` and the `$PATH`.

```
export LLVM_SYS_120_PREFIX=<your LLVM 12 install location>
export PATH=<your LLVM 12 install location>/bin:$PATH
```

Thirdly, install wasm target dependencies.

```
make install-rustc-wasm
```

To build everything, run:

```
make
```

After building, we can add the following command line parameters to use the KCL high-performance version:

```
kcl --target native main.k
```

To test, run:

```
make test
```

## Building and Testing in Docker

1. `make -C .. sh-in-docker`
2. `./run.sh -a build-kclvm` only once
3. `./run.sh -a update-kclvm`
4. `export PATH=$PATH:/root/kclvm/_build/dist/ubuntu/kclvm/bin`
5. `kcl ./samples/hello.k`
6. `kcl ./samples/hello.k --target native`
7. `cd kclvm && make test-grammar`

## IDE

You can choose any IDE you like for development, but we recommend a combination of [VS Code](https://code.visualstudio.com/) and the [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=matklad.rust-analyzer) plugin.

## Notes

1. If you encounter problems compiling KCLVM and using LLVM 12 on Apple M1, you can refer to the [documentation](./docs/m1-mac-setup.md)
2. If you wanna start over, you `MUST` clean up all cached building files, such as `LLVM build files`, `kclvm/target`, etc.
3. If your updating-cargo-index is extremely slow, setup `~/.cargo/config` file.

```
[source.crates-io]
registry = "https://github.com/rust-lang/crates.io-index"
replace-with = 'ustc'

[source.ustc]
registry = "git://mirrors.ustc.edu.cn/crates.io-index"
```
