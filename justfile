# Harp build commands

# Default recipe - show available commands
default:
    @just --list

# Build all Rust crates
build:
    cargo build --release

# Run all tests
test:
    cargo test

# Build and test everything
all: build test build-python build-wasm
    @echo "All builds complete"

# === CLI ===

# Build CLI
build-cli:
    cargo build --release -p harp-cli

# Build CLI with UPX compression (requires upx)
build-cli-compressed: build-cli
    mkdir -p dist
    cp target/release/harp dist/harp
    upx --best --lzma dist/harp
    @echo "Before/After:"
    @ls -lh target/release/harp dist/harp

# Install CLI locally
install-cli:
    cargo install --path crates/harp-cli

# Run harp
run:
    @cargo run --release -p harp-cli --quiet

# === Python ===

# Build Python bindings (requires maturin)
build-python:
    cd crates/harp-python && maturin build --release

# Build and install Python bindings in current venv
install-python:
    cd crates/harp-python && maturin develop

# Test Python bindings
test-python: install-python
    python -c "import harp; print(f'name={harp.generate_name()}'); print(f'version={harp.version()}')"

# === Go ===

# Build WASM core for Go bindings (WASI target)
build-wasm-core:
    cargo build --release -p harp-wasm-core --target wasm32-wasip1
    cp target/wasm32-wasip1/release/harp_wasm_core.wasm bindings/go/

# Build FFI library for Go
build-ffi:
    cargo build --release -p harp-ffi

# Test Go bindings (requires WASM core built first)
test-go: build-wasm-core
    cd bindings/go && go test -v

# === TypeScript/WASM ===

# Build WASM bindings (requires wasm-pack)
build-wasm:
    wasm-pack build crates/harp-wasm --target web --out-dir ../../bindings/typescript/pkg

# Build WASM for Node.js
build-wasm-node:
    wasm-pack build crates/harp-wasm --target nodejs --out-dir ../../bindings/typescript/pkg-node

# Test WASM bindings
test-wasm: build-wasm
    cd bindings/typescript && node --experimental-wasm-modules test.mjs

# === Docker ===

# Build Rust library in Docker (includes UPX compression)
docker-rust:
    docker compose run --rm rust

# Build Python wheel in Docker
docker-python:
    docker compose run --rm python

# Build and test Go bindings in Docker
docker-go:
    docker compose run --rm go

# Build and test WASM in Docker
docker-wasm:
    docker compose run --rm wasm

# Build CLI Docker image
docker-cli:
    docker compose build cli

# Build all in Docker
docker-all: docker-rust docker-python docker-go docker-wasm docker-cli
    @echo "All Docker builds complete"

# === Development ===

# Format all code
fmt:
    cargo fmt

# Run clippy
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# Check everything compiles
check:
    cargo check --all-targets

# Clean all build artifacts
clean:
    cargo clean
    rm -rf bindings/typescript/pkg bindings/typescript/pkg-node
    rm -rf crates/harp-ffi/include
    rm -rf dist

# === CI ===

# Run all checks (for CI)
ci: fmt lint test
    @echo "CI checks passed"

# === Publishing ===

# Publish harp-core to crates.io
publish-crate:
    cargo publish -p harp-core

# Publish all crates to crates.io (in order)
publish-crates:
    cargo publish -p harp-core
    @sleep 30  # Wait for crates.io to index
    cargo publish -p harp-ffi
    cargo publish -p harp-cli

# Dry run publish to crates.io
publish-crate-dry:
    cargo publish -p harp-core --dry-run

# Publish Python wheel to PyPI (requires maturin and MATURIN_PYPI_TOKEN)
publish-python:
    cd crates/harp-python && maturin publish

# Dry run Python publish
publish-python-dry:
    cd crates/harp-python && maturin build --release

# Publish WASM to npm (requires npm login)
publish-npm: build-wasm
    cd bindings/typescript && npm publish --access public

# Dry run npm publish
publish-npm-dry: build-wasm
    cd bindings/typescript && npm pack

# Tag a release (e.g., just tag v0.1.0)
tag VERSION:
    git tag -a {{VERSION}} -m "Release {{VERSION}}"
    git push origin {{VERSION}}
