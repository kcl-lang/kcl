# Build windows/amd64

1. install Git
1. install Rust and cargo
1. install Go1.16+
1. install VC2019
1. install TDM-GCC-x64
  - https://jmeubank.github.io/tdm-gcc/download/
1. install LLVM-12.0.1-win64
  - https://github.com/KusionStack/llvm-package-windows/releases/tag/v12.0.1
  - set `LLVM_SYS_120_PREFIX` to root path
1. Open Git Bash
  - `cd ./scripts/build-windows` and run `mingw32-make`
  - output: `scripts/build-windows/_output/kclvm-windows`

