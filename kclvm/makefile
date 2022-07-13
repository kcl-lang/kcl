default: run

PWD:=$(shell pwd)
COVER_REPORT_FILE_PATH:=$(PWD)/target/llvm-cov/html/index.html

# ------------------------
# Building and tesing deps
# ------------------------

# Generate runtime libraries when the runtime code is changed.
gen-runtime-api:
	make -C ./runtime gen-api-spec

# Install the wasm target
install-rustc-wasm:
	rustup target add wasm32-unknown-unknown

# Install python3 pytest
install-pytest:
	python3 -mpip install pytest pytest-html 

# ------------------------
# Compile and run
# ------------------------

# E2E compile and run
run:
	cd .. && ./run.sh -a update-kclvm && cd kclvm
	kcl ../samples/hello.k --target native

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
	cargo test -p kclvm-*

# Unit tests with code cov (Requires rust 1.60+)
codecov:
	rustup component add llvm-tools-preview
	cargo install cargo-llvm-cov
	cargo llvm-cov --html --open -p kclvm-*

# Unit tests with code cov and output the lcov file (Requires rust 1.60+)
codecov-lcov:
	rustup component add llvm-tools-preview
	cargo install cargo-llvm-cov
	mkdir $(PWD)/.kclvm
	cargo llvm-cov --lcov --output-path $(PWD)/.kclvm/lcov.info -p kclvm-*

# Test runtime libaries using python functions
test-runtime:
	cd ./tests/test_units && PYTHONPATH=./../../plugin kclvm -m pytest -vv || { echo 'kclvm/tests/test_units failed' ; exit 1; }

# E2E grammar tests.
test-grammar:
	cd tests/integration/grammar && kclvm -m pytest -v -n 5

# Parser fuzz.
fuzz-parser:
	cd tests && cargo fuzz run fuzz_parser
