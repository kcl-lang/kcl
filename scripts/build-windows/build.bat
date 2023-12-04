:: Copyright 2021 The KCL Authors. All rights reserved.

setlocal

cd %~dp0

:: install
call .\\build_kclvm_dll.bat
call .\\build_kclvm_cli.bat
call .\\build_kcl_lsp_server.bat

:: Copy C API header
call .\\build_kclvm_runtime.bat

:: install hello.k
copy ..\..\samples\hello.k .\_output\kclvm-windows\hello.k

_output\kclvm-windows\bin\kclvm_cli.exe run ..\..\samples\fib.k
_output\kclvm-windows\bin\kclvm_cli.exe run ..\..\samples\hello.k
_output\kclvm-windows\bin\kclvm_cli.exe run ..\..\samples\kubernetes.k
_output\kclvm-windows\bin\kclvm_cli.exe run ..\..\samples\math.k
