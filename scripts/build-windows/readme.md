# Build windows/amd64

1. install Rust and cargo
1. install Go1.16+
  - setup ssh key, keep `go get kcl-website` work well
1. install VC2019
1. install TDM-GCC-x64
  - https://jmeubank.github.io/tdm-gcc/download/
1. install LLVM-12.0.1-win64
  - https://github.com/PLC-lang/llvm-package-windows/releases/tag/v12.0.1
  - set `LLVM_SYS_120_PREFIX` to root path
2. install NSIS
  - https://nsis.sourceforge.io/Download
  - set `$PATH`
1. open VS2019-x64 command line
  - `cd kclvm` and run `cargo build` to check env
  - `cd ./scripts/build-windows` run `build.bat`
