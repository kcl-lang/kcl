# Copyright 2021 The KCL Authors. All rights reserved.

default:
	go install github.com/golang/protobuf/protoc-gen-go@latest

	mkdir -p ./_output

	# modfile
	protoc -I. --python_out=./_output ./modfile/modfile.proto

	cp ./_output/modfile/modfile_pb2.py \
		../config/modfile_pb2.py

	# gpyrpc
	protoc -I. --python_out=./_output ./gpyrpc/gpyrpc.proto ./gpyrpc/protorpc_wire.proto

	mkdir -p ../internal/gpyrpc
	cp ./_output/gpyrpc/gpyrpc_pb2.py \
		../internal/gpyrpc/gpyrpc_pb2.py
	cp ./_output/gpyrpc/protorpc_wire_pb2.py \
		../internal/gpyrpc/protorpc_wire_pb2.py

	# protorpc
	protoc -I. --python_out=./_output ./gpyrpc/protorpc_wire.proto
	make protorpc-stub

	-rm -rf ./_output

protorpc-stub:
	cd ./gpyrpc/protoc-gen-protorpc-py && go fmt \
		&& go install github.com/golang/protobuf/protoc-gen-go@latest \
		&& go install .

	cd gpyrpc && protoc --protorpc-py_out=. gpyrpc.proto
	cp ./gpyrpc/kusionstack.io/kclvm-go/pkg/spec/gpyrpc/gpyrpc.pb.protorpc.py \
		../internal/gpyrpc/gpyrpc_pb_protorpc.py

	-rm -rf ./gpyrpc/kusionstack.io

	cd .. && make format

clean:
	-rm -rf ./_output
