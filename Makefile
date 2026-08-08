.PHONY: build test run release help

help:
	@echo "targets: build test run release"

build:
	cargo build

test:
	cargo test

run:
	cargo run

release:
	cargo build --release
