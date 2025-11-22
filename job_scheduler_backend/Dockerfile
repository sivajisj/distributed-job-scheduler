FROM rust:1.70 as builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml .
COPY Cargo.lock .

# Create dummy source to build dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy real source code
COPY src ./src
COPY migrations ./migrations

# Build the application
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y openssl ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/job_scheduler_backend /app/backend

EXPOSE 8000

CMD ["./backend"]