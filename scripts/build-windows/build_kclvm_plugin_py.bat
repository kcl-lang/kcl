:: Copyright 2021 The KCL Authors. All rights reserved.

setlocal
cd %~dp0

:: install kclvm-plugin python module
cd ..\..\kclvm\plugin
go run .\copy-file.go -src=.\kclvm_plugin.py     -dst=..\..\scripts\build-windows\_output\kclvm-windows\lib\site-packages\kclvm_plugin.py
go run .\copy-file.go -src=.\kclvm_runtime.py    -dst=..\..\scripts\build-windows\_output\kclvm-windows\lib\site-packages\kclvm_runtime.py

cd %~dp0
go run .\copy-dir.go ..\..\plugins ..\..\scripts\build-windows\_output\kclvm-windows\plugins
cd %~dp0
