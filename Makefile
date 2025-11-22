# Copyright The KCL Authors. All rights reserved.

PROJECT_NAME = kcl
PWD:=$(shell pwd)

# ----------------
# Build
# ----------------

.PHONY: build
build:
	${PWD}/scripts/build.sh

.PHONY: build-wasm
build-wasm:
	cargo build --target=wasm32-wasip1 --release

.PHONY: build-lsp
build-lsp:
	cargo build --release --manifest-path crates/tools/src/LSP/Cargo.toml

.PHONY: build-cli
build-cli:
	cargo build --release --manifest-path crates/cli/Cargo.toml

.PHONY: release
release:
	${PWD}/scripts/release.sh

.PHONY: check
check:
	cargo check -r --all

.PHONY: fmt
fmt:
	cargo fmt --all

# Cargo clippy all packages
.PHONY: lint
lint:
	cargo clippy

# Cargo clippy all packages
.PHONY: lint-all
lint-all:
	cargo clippy --workspace --all-features --benches --examples --tests

# Cargo clippy all packages witj auto fix
.PHONY: fix
fix:
	cargo clippy --fix --allow-dirty

# Generate runtime libraries when the runtime code is changed.
gen-runtime-api:
	make -C crates/runtime gen-api-spec
	make fmt

# Install the wasm-wasi target
install-rustc-wasm-wasi:
	rustup target add wasm32-wasip1

# Install python3 pytest
install-test-deps:
	python3 -m pip install --user -U pytest pytest-html pytest-xdist ruamel.yaml

# ------------------------
# Tests
# ------------------------

# Unit tests without code cov
test:
	cargo test --workspace -r -- --nocapture

# Test runtime libaries using python functions
test-runtime: install-test-deps
	cd tests/runtime && PYTHONPATH=. python3 -m pytest -vv || { echo 'runtime test failed' ; exit 1; }

# E2E grammar tests with the fast evaluator
test-grammar: install-test-deps
	cd tests/grammar && python3 -m pytest -v -n 5
