VERSION=26.11.27.1




.PHONY: setup assets test lint build serve

setup:
	@echo "Setting up Rust and Python (uv) environments..."
	rustup target add wasm32-unknown-unknown
	cd scripts && uv sync

assets:
	@echo "Downloading assets using uv..."
	cd scripts && uv run download_assets.py

test:
	cargo test
	@echo "Running Python tests using uv/pytest..."
	cd scripts && uv run pytest

lint:
	@echo "Linting Python code using uv/ruff..."
	cd scripts && uv run ruff check .
	@echo "Linting Rust code..."
	cargo clippy -- -D warnings

build: assets
	@echo "Building Zookeeper WASM for version $(VERSION)..."
	cargo build --target wasm32-unknown-unknown --release
	@mkdir -p dist
	@# Try both underscored and hyphenated names as Cargo behavior can vary
	@if [ -f ../../../target/wasm32-unknown-unknown/release/zookeeper_wasm.wasm ]; then \
		cp ../../../target/wasm32-unknown-unknown/release/zookeeper_wasm.wasm dist/; \
	elif [ -f ../../../target/wasm32-unknown-unknown/release/zookeeper-wasm.wasm ]; then \
		cp ../../../target/wasm32-unknown-unknown/release/zookeeper-wasm.wasm dist/zookeeper_wasm.wasm; \
	elif [ -f ../../target/wasm32-unknown-unknown/release/zookeeper_wasm.wasm ]; then \
		cp ../../target/wasm32-unknown-unknown/release/zookeeper_wasm.wasm dist/; \
	elif [ -f ../../target/wasm32-unknown-unknown/release/zookeeper-wasm.wasm ]; then \
		cp ../../target/wasm32-unknown-unknown/release/zookeeper-wasm.wasm dist/zookeeper_wasm.wasm; \
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
