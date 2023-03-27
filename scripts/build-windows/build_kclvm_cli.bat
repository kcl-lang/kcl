:: Copyright 2021 The KCL Authors. All rights reserved.

setlocal
cd %~dp0

:: install kclvm-cli
cd ..\..\kclvm_cli
cargo build --release
cd %~dp0

go run .\copy-file.go --src=..\..\kclvm_cli\target\release\kclvm_cli.exe --dst=.\_output\kclvm-windows\bin\kclvm-cli.exe

