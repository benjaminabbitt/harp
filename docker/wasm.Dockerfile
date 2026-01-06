# Build WASM bindings using wasm-pack
FROM rustlang/rust:nightly-bookworm

# Install wasm-pack and Node.js for testing
RUN apt-get update && apt-get install -y \
    nodejs \
    npm \
    && rm -rf /var/lib/apt/lists/*

RUN cargo install wasm-pack

WORKDIR /workspace

# Build WASM and run tests
CMD wasm-pack build crates/harp-wasm --target web --out-dir ../../bindings/typescript/pkg && \
    cd bindings/typescript && \
    node --experimental-wasm-modules test.mjs
