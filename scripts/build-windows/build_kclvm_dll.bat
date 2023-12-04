:: Copyright 2021 The KCL Authors. All rights reserved.

setlocal
cd %~dp0

:: install kclvm_cli_cdylib.dll
cd ..\..\kclvm
cargo build --release
cd %~dp0

copy ..\..\kclvm\target\release\kclvm_cli_cdylib.dll .\_output\kclvm-windows\bin\kclvm_cli_cdylib.dll
copy ..\..\kclvm\target\release\kclvm_cli_cdylib.dll.lib .\_output\kclvm-windows\bin\kclvm_cli_cdylib.lib
copy .\copy-file.go -src=..\..\kclvm\target\release\kclvm_cli_cdylib.dll.lib ..\..\kclvm\target\release\kclvm_cli_cdylib.lib

:: install hello.k
copy ..\..\samples\hello.k .\_output\kclvm-windows\hello.k
