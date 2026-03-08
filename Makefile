# J-Law-Core Makefile
#
# Run all CI checks with a single command: make ci

.PHONY: all fmt fmt-check clippy test audit ci docker-test test-python

all: ci

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

clippy:
	cargo clippy --workspace -- -D warnings

test:
	cargo test --workspace

audit:
	cargo audit

ci: fmt-check clippy test test-python

test-python:
	pip3 install -q maturin pytest
	maturin build -m crates/j-law-uniffi/Cargo.toml
	pip3 install -q --break-system-packages target/wheels/*.whl
	pip3 install -q --break-system-packages crates/j-law-python/
	python3 -m pytest crates/j-law-python/tests/ -v

docker-test:
	docker compose up test-all --build
