PWD:=$(shell pwd)

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
