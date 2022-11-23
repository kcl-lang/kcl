# Copyright 2021 The KCL Authors. All rights reserved.

# Only on windows platform: use mingw32-make to run.

default:
	go run download-file.go
	go run unzip.go

	go run gen_pth.go

	# install pip
	_output/kclvm-windows/python.exe get-pip.py

	# pip install kclvm
	_output/kclvm-windows/python.exe -m pip install kclvm

	# go run gen-kclvm-py.go

	# renname
	go run rename.go -old="_output/kclvm-windows/python.exe" -new="_output/kclvm-windows/kclvm.exe"

	# install python39 include and libs
	go run ./copy-dir.go ./py39-libs ./_output/kclvm-windows

	# install kclvm-runtime
	cd ../../kclvm/runtime && cargo build --release
	go run ./copy-file.go -src=../../kclvm/target/release/kclvm.dll             -dst=./_output/kclvm-windows/libs/kclvm.dll
	go run ./copy-file.go -src=../../kclvm/target/release/kclvm.dll.lib         -dst=./_output/kclvm-windows/libs/kclvm.dll.lib
	go run ./copy-file.go -src=../../kclvm/runtime/src/_kclvm.ll                -dst=./_output/kclvm-windows/libs/_kclvm.ll
	go run ./copy-file.go -src=../../kclvm/runtime/src/_kclvm.bc                -dst=./_output/kclvm-windows/libs/_kclvm.bc
	go run ./copy-file.go -src=../../kclvm/runtime/src/_kclvm.h                 -dst=./_output/kclvm-windows/libs/_kclvm.h
	go run ./copy-file.go -src=../../kclvm/runtime/src/_kclvm_main_win.c        -dst=./_output/kclvm-windows/libs/_kclvm_main_win.c

	# install kclvm-plugin
	./_output/kclvm-windows/kclvm.exe ../../kclvm/plugin/setup.py install_lib

	# install kclvm-cli
	cd ../../kclvm && cargo build --release
	go run ./copy-file.go -src=../../kclvm/target/release/kclvm_cli.exe            -dst=./_output/kclvm-windows/kclvm_cli.exe
	go run ./copy-file.go -src=../../kclvm/target/release/kclvm_cli_cdylib.dll     -dst=./_output/kclvm-windows/kclvm_cli_cdylib.dll
	go run ./copy-file.go -src=../../kclvm/target/release/kclvm_cli_cdylib.dll.lib -dst=./_output/kclvm-windows/libs/kclvm_cli_cdylib.dll.lib

	# build rust std lib
	go run install-rust-std.go -outdir=./_output/kclvm-windows

	# install kcl plugins
	go run ./copy-dir.go ../../plugins ./_output/kclvm-windows/plugins

	# install hello.k
	go run ./copy-file.go -src=../../samples/hello.k -dst=./_output/kclvm-windows/hello.k

	# install tools
	go build -o ./_output/kclvm-windows/kcl.exe        kcl.go
	go build -o ./_output/kclvm-windows/kcl-doc.exe    kcl-doc.go
	go build -o ./_output/kclvm-windows/kcl-lint.exe   kcl-lint.go
	go build -o ./_output/kclvm-windows/kcl-fmt.exe    kcl-fmt.go
	go build -o ./_output/kclvm-windows/kcl-plugin.exe kcl-plugin.go
	go build -o ./_output/kclvm-windows/kcl-vet.exe    kcl-vet.go
	go build -o ./_output/kclvm-windows/kcl-fmt.exe    kcl-fmt.go

	# todo: build zip
	
	# todo: run hello.k
	# _output/kclvm-windows/kclvm.exe -m kclvm ../../samples/hello.k
	# _output/kclvm-windows/kcl-go.exe run     ../../samples/hello.k
	# _output/kclvm-windows/kcl.exe            ../../samples/hello.k


clean:

