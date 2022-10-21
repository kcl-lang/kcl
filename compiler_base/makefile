PWD:=$(shell pwd)
COVER_REPORT_FILE_PATH:=$(PWD)/target/llvm-cov/html/index.html

# ------------------------
# Compile and run
# ------------------------

# Cargo check all packages
check:
	cargo check --release

# Cargo fmt all packages
fmt:
	cargo fmt --all

# Cargo clippy all packages
lint:
	cargo clippy

# ------------------------
# Tests
# ------------------------

# Unit tests without code cov
test:
	cargo test -p compiler_base_*

# Unit tests with code cov (Requires rust 1.60+)
codecov:
	rustup component add llvm-tools-preview
	cargo install cargo-llvm-cov
	cargo llvm-cov --workspace --ignore-filename-regex gpyrpc.rs --html --open 

# Unit tests with code cov and output the lcov file (Requires rust 1.60+)
codecov-lcov:
	rustup component add llvm-tools-preview
	cargo install cargo-llvm-cov
	mkdir $(PWD)/.compiler_base
	cargo llvm-cov --lcov --output-path $(PWD)/.compiler_base/lcov.info --workspace --ignore-filename-regex gpyrpc.rs
