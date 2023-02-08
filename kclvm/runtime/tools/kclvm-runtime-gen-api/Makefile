default:
	go run main.go \
		-root=../../src \
		-c-api=../../src/_kclvm.h \
		-ll-api=../../src/_kclvm.ll \
		-rust-api-enum=../../src/_kclvm.rs \
		-rust-api-addr=../../src/_kclvm_addr.rs

	llvm-as ../../src/_kclvm.ll
