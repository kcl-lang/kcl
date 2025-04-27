default: run

PWD:=$(shell pwd)
COVER_REPORT_FILE_PATH:=$(PWD)/target/llvm-cov/html/index.html

# ------------------------
# Building and tesing deps
# ------------------------

# Generate runtime libraries when the runtime code is changed.
gen-runtime-api:
	make -C ./runtime gen-api-spec
	make fmt

# Install the wasm32-unknown-unknown target
install-rustc-wasm:
	rustup target add wasm32-unknown-unknown

# Install the wasm-wasi target
install-rustc-wasm-wasi:
	rustup target add wasm32-wasip1

# Install python3 pytest
install-test-deps:
	python3 -mpip install --user -U pytest pytest-html pytest-xdist ruamel.yaml

# ------------------------
# Compile and run
# ------------------------

# E2E compile and run
run:
	cd .. && make build && cd kclvm
	kclvm_cli run ../samples/hello.k

# Cargo check all packages
check:
	cargo check --release

# Cargo fmt all packages
fmt:
	cargo fmt --all

# Cargo clippy all packages
lint:
	cargo clippy

# Cargo clippy all packages witj auto ix
fix:
	cargo clippy --fix --allow-dirty

build-wasm:
	cargo build --target=wasm32-wasip1 --release

build-lsp:
	cargo build --release --manifest-path tools/src/LSP/Cargo.toml

# ------------------------
# Tests
# ------------------------

# Unit tests without code cov
test:
	cargo test --workspace -r -- --nocapture

# Unit tests with code cov (Requires rust 1.60+)
codecov:
	rustup component add llvm-tools-preview
	cargo install cargo-llvm-cov@0.6.15
	cargo llvm-cov --workspace --ignore-filename-regex gpyrpc.rs --html --open -- --nocapture

# Unit tests with code cov and output the lcov file (Requires rust 1.60+)
codecov-lcov:
	rustup component add llvm-tools-preview
	cargo install cargo-llvm-cov@0.6.15
	rm -rf $(PWD)/.kclvm_cov
	mkdir $(PWD)/.kclvm_cov
	cargo llvm-cov --lcov --output-path $(PWD)/.kclvm_cov/lcov.info -r --workspace --ignore-filename-regex gpyrpc.rs -- --nocapture

# Test runtime libaries using python functions
test-runtime: install-test-deps
	cd ./tests/test_units && PYTHONPATH=./../../tests/test_units/runtime python3 -m pytest -vv || { echo 'kclvm/tests/test_units failed' ; exit 1; }

# E2E grammar tests.
test-grammar: install-test-deps
	cd tests/integration/grammar && python3 -m pytest -v -n 5

# E2E grammar tests with the fast evaluator
test-grammar-evaluator: install-test-deps
	cd tests/integration/grammar && KCL_FAST_EVAL=1 python3 -m pytest -v -n 5

# E2E konfig tests.
test-konfig: install-test-deps
	cd tests/integration/konfig && python3 -m pytest -v -n 5

# Parser fuzz.
fuzz-parser:
	cd tests && cargo fuzz run fuzz_parser
