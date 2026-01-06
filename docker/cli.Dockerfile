# Build CLI binary
FROM rustlang/rust:nightly-bookworm AS builder

WORKDIR /workspace
COPY . .

RUN cargo build --release -p harp-cli

# Runtime image
FROM debian:bookworm-slim

COPY --from=builder /workspace/target/release/harp /usr/local/bin/harp

ENTRYPOINT ["harp"]
