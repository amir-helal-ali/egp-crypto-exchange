# syntax=docker/dockerfile:1.6

# ============================================================================
# Stage 1 — Builder
# ============================================================================
FROM rust:1.85-slim-bookworm AS builder

# Install required build dependencies for rustls + native libs
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Cache dependencies: copy Cargo files first, then build a dummy crate.
COPY Cargo.toml Cargo.lock* ./
RUN mkdir -p src && echo "fn main() {}" > src/main.rs
RUN cargo build --release || true

# Now copy the actual source and build the real binary.
COPY src ./src
COPY migrations ./migrations

# Touch main.rs so cargo rebuilds with the real source.
RUN touch src/main.rs

ENV SQLX_OFFLINE=true
RUN cargo build --release --bin egp-exchange-backend

# ============================================================================
# Stage 2 — Runtime (Distroless)
# ============================================================================
FROM gcr.io/distroless/cc-debian12:nonroot AS runtime

LABEL org.opencontainers.image.title="egp-exchange-backend" \
      org.opencontainers.image.description="EGP closed-loop cryptocurrency exchange — Rust backend" \
      org.opencontainers.image.source="https://github.com/egp-exchange/backend" \
      org.opencontainers.image.licenses="Proprietary"

WORKDIR /app

# Copy binary + migrations from builder.
COPY --from=builder /build/target/release/egp-exchange-backend /app/egp-exchange-backend
COPY --from=builder /build/migrations /app/migrations

# Distroless provides CA certs in the base image; ensure they're discoverable.
ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt \
    SSL_CERT_DIR=/etc/ssl/certs

# Non-root user (provided by distroless `nonroot` variant: uid=65532).
USER nonroot:nonroot

EXPOSE 8080

# Distroless has no shell — entrypoint must be the binary directly.
ENTRYPOINT ["/app/egp-exchange-backend"]
