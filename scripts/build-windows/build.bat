:: Copyright 2021 The KCL Authors. All rights reserved.

setlocal

cd %~dp0

:: install kclvm-cli
call .\\build_kclvm_dll.bat
call .\\build_kclvm_cli.bat
call .\\build_kcl_lsp_server.bat

:: install kclvm-plugin python module
call .\\build_kclvm_plugin_py.bat

:: Copy KCLVM C API header
call .\\build_kclvm_runtime.bat

:: install hello.k
go run .\copy-file.go --src=..\..\samples\hello.k --dst=.\_output\kclvm-windows\hello.k

:: install tools
go build -o .\_output\kclvm-windows\bin\kcl.exe        kcl.go
go build -o .\_output\kclvm-windows\bin\kclvm.exe      kclvm.go
go build -o .\_output\kclvm-windows\bin\kcl-plugin.exe kcl-plugin.go
go build -o .\_output\kclvm-windows\bin\kcl-doc.exe    kcl-doc.go
go build -o .\_output\kclvm-windows\bin\kcl-test.exe   kcl-test.go
go build -o .\_output\kclvm-windows\bin\kcl-lint.exe   kcl-lint.go
go build -o .\_output\kclvm-windows\bin\kcl-fmt.exe    kcl-fmt.go
go build -o .\_output\kclvm-windows\bin\kcl-vet.exe    kcl-vet.go

:: run hello.k
_output\kclvm-windows\bin\kcl.exe           ..\..\samples\fib.k
_output\kclvm-windows\bin\kcl.exe           ..\..\samples\hello.k
_output\kclvm-windows\bin\kcl.exe           ..\..\samples\kubernetes.k
_output\kclvm-windows\bin\kcl.exe           ..\..\samples\math.k

_output\kclvm-windows\bin\kclvm-cli.exe run ..\..\samples\fib.k
_output\kclvm-windows\bin\kclvm-cli.exe run ..\..\samples\hello.k
_output\kclvm-windows\bin\kclvm-cli.exe run ..\..\samples\kubernetes.k
_output\kclvm-windows\bin\kclvm-cli.exe run ..\..\samples\math.k
