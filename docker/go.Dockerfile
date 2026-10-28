# Build Go bindings with CGO
FROM rustlang/rust:nightly-bookworm

# Go from the official image, pinned to an exact patch of the bindings' floor
# (go 1.22, wazero v1.9.0). Debian's golang-go is Go 1.19, below that floor.
COPY --from=golang:1.22.12-bookworm /usr/local/go /usr/local/go
ENV PATH="/usr/local/go/bin:${PATH}"

WORKDIR /workspace

# Build FFI library and run Go tests
CMD cargo build --release -p harp-ffi && \
    cd bindings/go && \
    CGO_ENABLED=1 go test -v
