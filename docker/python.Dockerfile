# Build Python wheels using maturin
FROM rustlang/rust:nightly-bookworm

# Install Python and pip
RUN apt-get update && apt-get install -y \
    python3 \
    python3-pip \
    python3-venv \
    && rm -rf /var/lib/apt/lists/*

# Install maturin
RUN pip3 install maturin --break-system-packages

WORKDIR /workspace

# Build command
CMD ["maturin", "build", "--release", "-m", "crates/harp-python/Cargo.toml", "-o", "dist/python"]
