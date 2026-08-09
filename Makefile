.PHONY: build test run release lint help

help:
	@echo "targets: build test run release lint"

build:
	cargo build

test:
	cargo test

run:
	cargo run

release:
	cargo build --release

lint:
	cargo clippy --all-targets
