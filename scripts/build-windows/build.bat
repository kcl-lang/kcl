:: Copyright 2021 The KCL Authors. All rights reserved.

setlocal

cd %~dp0

go run download-file.go
go run unzip.go

go run gen_pth.go

:: renname
go run rename.go -old="_output\kclvm-windows\python.exe" -new="_output\kclvm-windows\kclvm.exe"

:: install pip
_output\kclvm-windows\kclvm.exe get-pip.py

:: pip install -r ..\requirements.txt
_output\kclvm-windows\kclvm.exe -m pip install ^
    -r .\requirements.release.txt ^
    --target=_output\kclvm-windows\Lib\site-packages

:: install kclvm
go run gen-kclvm-py.go

:: install python39 include and libs
go run .\copy-dir.go .\py39-libs .\_output\kclvm-windows

:: install kclvm-runtime
cd ..\..\kclvm\runtime
cargo build --release
cd %~dp0

go run .\copy-file.go --src=..\..\kclvm\runtime\target\release\kclvm.dll     --dst=.\_output\kclvm-windows\kclvm.dll
go run .\copy-file.go --src=..\..\kclvm\runtime\target\release\kclvm.dll.lib --dst=.\_output\kclvm-windows\libs\kclvm.dll.lib
go run .\copy-file.go --src=..\..\kclvm\runtime\src\_kclvm.ll                --dst=.\_output\kclvm-windows\libs\_kclvm.ll
go run .\copy-file.go --src=..\..\kclvm\runtime\src\_kclvm.bc                --dst=.\_output\kclvm-windows\libs\_kclvm.bc
go run .\copy-file.go --src=..\..\kclvm\runtime\src\_kclvm.h                 --dst=.\_output\kclvm-windows\libs\_kclvm.h
go run .\copy-file.go --src=..\..\kclvm\runtime\src\_kclvm_main_win.c        --dst=.\_output\kclvm-windows\libs\_kclvm_main_win.c

:: install kclvm-runtime (wasm)
cd ..\..\kclvm\runtime
cargo build --release --target=wasm32-unknown-unknown-wasm
cd %~dp0

go run .\copy-file.go --src=..\..\kclvm\runtime\target\wasm32-unknown-unknown\release\libkclvm.a --dst=.\_output\kclvm-windows\libs\libkclvm_wasm32.a
go run .\copy-file.go --src=..\..\kclvm\runtime\src\_kclvm_undefined_wasm.txt --dst=.\_output\kclvm-windows\libs\_kclvm_undefined_wasm.txt

:: install kclvm-plugin
.\_output\kclvm-windows\kclvm.exe ..\..\kclvm\plugin\setup.py install_lib
go run .\copy-file.go --src=..\..\kclvm\plugin\kclvm_plugin.py  --dst=.\_output\kclvm-windows\Lib\site-packages\kclvm_plugin.py
go run .\copy-file.go --src=..\..\kclvm\plugin\kclvm_runtime.py --dst=.\_output\kclvm-windows\Lib\site-packages\kclvm_runtime.py

:: install kclvm-cli
cd ..\..\kclvm
cargo build --release
cd %~dp0

go run .\copy-file.go --src=..\..\kclvm\target\release\kclvm.exe --dst=.\_output\kclvm-windows\kclvm-cli.exe

:: install clang
go run .\copy-file.go ^
    --src=%LLVM_SYS_120_PREFIX%\bin\clang.exe ^
    --dst=.\_output\kclvm-windows\tools\clang\bin\clang.exe

:: install hello.k
go run .\copy-file.go --src=..\..\hello.k --dst=.\_output\kclvm-windows\hello.k

:: install tools
go build -o .\_output\kclvm-windows\kcl.exe        kcl.go
go build -o .\_output\kclvm-windows\kcl-doc.exe    kcl-doc.go
go build -o .\_output\kclvm-windows\kcl-lint.exe   kcl-lint.go
go build -o .\_output\kclvm-windows\kcl-fmt.exe    kcl-fmt.go
go build -o .\_output\kclvm-windows\kcl-plugin.exe kcl-plugin.go
go build -o .\_output\kclvm-windows\kcl-vet.exe    kcl-vet.go

:: run hello.k
_output\kclvm-windows\kclvm.exe -m kclvm ..\..\hello.k
_output\kclvm-windows\kcl-go.exe run     ..\..\hello.k
_output\kclvm-windows\kcl.exe            ..\..\hello.k

