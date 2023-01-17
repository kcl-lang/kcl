:: Copyright 2021 The KCL Authors. All rights reserved.

setlocal
cd %~dp0

:: Copy KCLVM C API header
cd ..\..\kclvm\runtime
go run .\copy-file.go --src=src\_kclvm.h --dst=..\..\scripts\build-windows\_output\kclvm-windows\include\_kclvm.h
cd %~dp0
