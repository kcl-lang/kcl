default:
	cargo run --release

llvm:
	clang++ -O3 -emit-llvm ./C/test.c -S -o ./C/test.ll
	llvm-as ./C/test.ll
	lli ./C/test.bc

fmt:
	cargo fmt

check:
	cargo check --release

lint:
	cargo clippy
