:: Copyright 2021 The KCL Authors. All rights reserved.

setlocal
cd %~dp0

:: install kclvm_cli_cdylib.dll
cd ..\..\kclvm
cargo build --release
cd %~dp0

go run .\copy-file.go -src=..\..\kclvm\target\release\kclvm_cli_cdylib.dll     -dst=.\_output\kclvm-windows\bin\kclvm_cli_cdylib.dll
go run .\copy-file.go -src=..\..\kclvm\target\release\kclvm_cli_cdylib.dll.lib -dst=.\_output\kclvm-windows\bin\kclvm_cli_cdylib.lib
go run .\copy-file.go -src=..\..\kclvm\target\release\kclvm_cli_cdylib.dll.lib -dst=..\..\kclvm\target\release\kclvm_cli_cdylib.lib

:: install hello.k
go run .\copy-file.go --src=..\..\samples\hello.k --dst=.\_output\kclvm-windows\hello.k

