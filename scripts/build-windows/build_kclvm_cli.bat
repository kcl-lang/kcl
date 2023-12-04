:: Copyright 2021 The KCL Authors. All rights reserved.

setlocal
cd %~dp0

:: install kclvm CLI
cd ..\..\cli
cargo build --release
cd %~dp0

copy ..\..\kclvm_cli\target\release\kclvm_cli.exe .\_output\kclvm-windows\bin\kclvm_cli.exe

