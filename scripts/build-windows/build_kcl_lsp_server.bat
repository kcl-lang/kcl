:: Copyright 2021 The KCL Authors. All rights reserved.

setlocal
cd %~dp0

:: install kclvm-cli
cd ..\..\kclvm\tools\src\LSP
cargo build --release
cd %~dp0

go run .\copy-file.go --src=..\..\kclvm\target\release\kcl-language-server.exe --dst=.\_output\kclvm-windows\bin\kcl-language-server.exe
