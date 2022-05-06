default: run

PWD:=$(shell pwd)
RUNTIME_NATIVE_BC_PATH:=./runtime/src/_kclvm.bc
COVER_TEST_FILE_PATH:=$(PWD)/../cover/TEST.xml
COVER_REPORT_FILE_PATH:=$(PWD)/../cover/report.html
CARGO_LIBS = $(shell find . -maxdepth 1 -type d) 

run:
	cd .. && ./run.sh -a update-kclvm && cd kclvm
	kcl ../samples/hello.k --target native

gen-runtime-api:
	make -C ./runtime gen-api-spec

install-rustc-wasm:
	rustup target add wasm32-unknown-unknown

check:
	cargo check --release

fmt:
	cargo fmt

test:
	cargo test

test-grammar: install-pytest
	cd tests/integration/grammar && kclvm -m pytest -v -n 5 --junitxml $(COVER_TEST_FILE_PATH) --html=$(COVER_REPORT_FILE_PATH)

open-test-report:
	open $(COVER_REPORT_FILE_PATH)

test_units:
	cd ./runtime && cargo build --release
	kclvm -m pytest tests/test_units -vv

install-pytest:
	kclvm -mpip install pytest-html 

release:
	cargo build --release

lint:
	@for i in $(CARGO_LIBS); do cd $(PWD)/$$i && cargo clippy && cd ..; done  
	@echo lint finished!
