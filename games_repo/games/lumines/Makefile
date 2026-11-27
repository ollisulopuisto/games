VERSION=26.11.27.2

.PHONY: build serve test lint

test:
	@echo "Running tests..."
	cargo test

lint:
	@echo "Running clippy..."
	cargo clippy -- -D warnings

build:
	@echo "Building Lumines WASM for version $(VERSION)..."
	cargo build --target wasm32-unknown-unknown --release
	@mkdir -p dist
	@if [ -f ../../../target/wasm32-unknown-unknown/release/lumines_wasm.wasm ]; then \
		cp ../../../target/wasm32-unknown-unknown/release/lumines_wasm.wasm dist/; \
	elif [ -f ../../../target/wasm32-unknown-unknown/release/lumines-wasm.wasm ]; then \
		cp ../../../target/wasm32-unknown-unknown/release/lumines-wasm.wasm dist/lumines_wasm.wasm; \
	elif [ -f ../../target/wasm32-unknown-unknown/release/lumines_wasm.wasm ]; then \
		cp ../../target/wasm32-unknown-unknown/release/lumines_wasm.wasm dist/; \
	elif [ -f ../../target/wasm32-unknown-unknown/release/lumines-wasm.wasm ]; then \
		cp ../../target/wasm32-unknown-unknown/release/lumines-wasm.wasm dist/lumines_wasm.wasm; \
	else \
		echo "Error: WASM binary not found in target directory"; exit 1; \
	fi
	cp index.html dist/
	@if [ ! -f dist/mq_js_bundle.js ]; then \
		curl -sO https://not-fl3.github.io/miniquad-samples/mq_js_bundle.js && mv mq_js_bundle.js dist/; \
	fi

serve: build
	@echo "Serving locally at http://localhost:8001"
	python3 -m http.server -d dist 8001
