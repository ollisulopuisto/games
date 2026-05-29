VERSION=26.4.30.256

.PHONY: setup test lint build serve

setup:
	@echo "Setting up Rust environment..."
	rustup target add wasm32-unknown-unknown

test:
	cargo test

lint:
	@echo "Linting Rust code..."
	cargo clippy -- -D warnings

build:
	@echo "Building Kissagotchi WASM for version $(VERSION)..."
	cargo build --target wasm32-unknown-unknown --release
	@mkdir -p dist
	@if [ -f ../../../target/wasm32-unknown-unknown/release/kissagotchi_wasm.wasm ]; then \
		cp ../../../target/wasm32-unknown-unknown/release/kissagotchi_wasm.wasm dist/; \
	elif [ -f ../../../target/wasm32-unknown-unknown/release/kissagotchi-wasm.wasm ]; then \
		cp ../../../target/wasm32-unknown-unknown/release/kissagotchi-wasm.wasm dist/kissagotchi_wasm.wasm; \
	elif [ -f ../../target/wasm32-unknown-unknown/release/kissagotchi_wasm.wasm ]; then \
		cp ../../target/wasm32-unknown-unknown/release/kissagotchi_wasm.wasm dist/; \
	elif [ -f ../../target/wasm32-unknown-unknown/release/kissagotchi-wasm.wasm ]; then \
		cp ../../target/wasm32-unknown-unknown/release/kissagotchi-wasm.wasm dist/kissagotchi_wasm.wasm; \
	else \
		echo "Error: WASM binary not found in target directory"; exit 1; \
	fi
	cp index.html dist/
	@if [ ! -f dist/mq_js_bundle.js ]; then \
		curl -sO https://not-fl3.github.io/miniquad-samples/mq_js_bundle.js && mv mq_js_bundle.js dist/; \
	fi

serve: build
	@echo "Serving locally at http://localhost:8000"
	python3 -m http.server -d dist
