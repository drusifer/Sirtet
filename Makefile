.PHONY: build test run release lint web wasm serve help

help:
	@echo "targets: build test run release lint web wasm serve"

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

web: wasm

wasm:
	cargo build --target wasm32-unknown-unknown --release
	mkdir -p web
	cp target/wasm32-unknown-unknown/release/tetris.wasm web/
	cp "$$(cargo metadata --format-version 1 | python3 -c 'import json,sys; pkgs=json.load(sys.stdin)["packages"]; print([p for p in pkgs if p["name"]=="miniquad"][0]["manifest_path"])' | xargs dirname)/js/gl.js" web/mq_js_bundle.js

serve: wasm
	python3 -m http.server 8080 --directory web

