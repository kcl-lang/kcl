:: Copyright 2021 The KCL Authors. All rights reserved.

setlocal
cd %~dp0

:: Copy KCLVM C API header
go run .\copy-file.go --src=..\..\kclvm\runtime\src\_kclvm.h --dst=.\_output\kclvm-windows\include\_kclvm.h
cd %~dp0
