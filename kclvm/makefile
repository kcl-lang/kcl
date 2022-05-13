default: run

PWD:=$(shell pwd)
RUNTIME_NATIVE_BC_PATH:=./runtime/src/_kclvm.bc
COVER_TEST_FILE_PATH:=$(PWD)/../cover/TEST.xml
COVER_REPORT_FILE_PATH:=$(PWD)/../cover/report.html
CARGO_LIBS = $(shell find . -maxdepth 1 -type d) 

run:
	cd .. && ./run.sh -a update-kclvm && cd kclvm
	kcl ../samples/hello.k --target native

check:
	cargo check --release

fmt:
	cargo fmt --all

lint:
	@for i in $(CARGO_LIBS); do cd $(PWD)/$$i && cargo clippy && cd ..; done

test:
	@for i in $(CARGO_LIBS); do cd $(PWD)/$$i && cargo test || { echo 'test kclvm/' $$i 'crate failed' ; exit 1; } && cd ..; done

test-runtime:
	cd ./tests/test_units && python3 -m pip install pytest && PYTHONPATH=./../../plugin python3 -m pytest -vv || { echo 'kclvm/tests/test_units failed' ; exit 1; }

test-grammar: install-pytest
	cd tests/integration/grammar && kclvm -m pytest -v -n 5 --junitxml $(COVER_TEST_FILE_PATH) --html=$(COVER_REPORT_FILE_PATH)

install-pytest:
	kclvm -mpip install pytest-html 

release:
	cargo build --release


gen-runtime-api:
	make -C ./runtime gen-api-spec

install-rustc-wasm:
	rustup target add wasm32-unknown-unknown
