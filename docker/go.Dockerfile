# Build Go bindings with CGO
FROM rustlang/rust:nightly-bookworm

# Install Go
RUN apt-get update && apt-get install -y \
    golang-go \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace

# Build FFI library and run Go tests
CMD cargo build --release -p harp-ffi && \
    cd bindings/go && \
    CGO_ENABLED=1 go test -v
