# Build Rust library and run tests
FROM rustlang/rust:nightly-bookworm

# Install UPX for binary compression
RUN curl -L https://github.com/upx/upx/releases/download/v4.2.4/upx-4.2.4-amd64_linux.tar.xz | \
    tar -xJ --strip-components=1 -C /usr/local/bin upx-4.2.4-amd64_linux/upx

WORKDIR /workspace

# Build and test
CMD cargo build --release && cargo test && \
    mkdir -p dist && \
    cp target/release/harp dist/harp && \
    upx --best --lzma dist/harp && \
    echo "Build complete:" && \
    ls -lh target/release/harp dist/harp
