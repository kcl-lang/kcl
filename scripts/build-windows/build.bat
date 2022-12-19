:: Copyright 2021 The KCL Authors. All rights reserved.

setlocal

cd %~dp0

go run download-file.go
go run unzip.go

go run gen_pth.go

:: renname
go run rename.go -old="_output\python.exe" -new="_output\kclvm_py.exe"

:: install pip
_output\kclvm_py.exe get-pip.py

:: pip install -r ..\requirements.txt
_output\kclvm_py.exe -m pip install ^
    -r .\requirements.release.txt ^
    --target=_output\Lib\site-packages

:: install kclvm
go run gen-kclvm-py.go

:: install python39 include and libs
go run .\copy-dir.go .\py39-libs .\_output

:: install kclvm-cli
call .\\build_kclvm_cli.bat

:: install hello.k
go run .\copy-file.go --src=..\..\samples\hello.k --dst=.\_output\hello.k

:: install tools
go build -o .\_output\kcl.exe        kcl.go
go build -o .\_output\kcl-doc.exe    kcl-doc.go
go build -o .\_output\kcl-lint.exe   kcl-lint.go
go build -o .\_output\kcl-fmt.exe    kcl-fmt.go
go build -o .\_output\kcl-plugin.exe kcl-plugin.go
go build -o .\_output\kcl-vet.exe    kcl-vet.go

:: run hello.k
_output\kcl.exe            ..\..\hello.k
_output\kclvm-cli.exe run ..\..\samples\hello_invalid.k
_output\kclvm-cli.exe run ..\..\samples\hello.k

