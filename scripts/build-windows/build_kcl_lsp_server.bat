:: Copyright 2021 The KCL Authors. All rights reserved.

setlocal
cd %~dp0

:: install kcl language server
cd ..\..\kclvm\tools\src\LSP
cargo build --release
cd %~dp0

copy ..\..\kclvm\target\release\kcl-language-server.exe .\_output\kclvm-windows\bin\kcl-language-server.exe
