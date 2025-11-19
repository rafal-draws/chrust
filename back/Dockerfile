# Build stage
FROM rust:1.88 AS builder

WORKDIR /usr/src/myapp

# Install build dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends dos2unix wget unzip && \
    rm -rf /var/lib/apt/lists/*

# Download and setup libtorch
COPY libtorch_setup.sh .
RUN dos2unix libtorch_setup.sh && bash ./libtorch_setup.sh

ENV LIBTORCH=/home/util/libtorch/libtorch
ENV LIBTORCH_INCLUDE=$LIBTORCH
ENV LIBTORCH_LIB=$LIBTORCH
ENV LD_LIBRARY_PATH=$LIBTORCH/lib:/usr/local/lib:/usr/lib

# Set DATABASE_URL for SQLx compile-time verification
ENV DATABASE_URL=postgres://psqlx:psqlx@postgres:5432/uploads
# Allow SQLx to work offline if database is not available
ENV SQLX_OFFLINE=true

# Copy source and build
COPY . .
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install only runtime dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends libgomp1 ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy libtorch libraries from builder (needed at runtime)
COPY --from=builder /home/util/libtorch/libtorch/lib /usr/local/lib/libtorch

# Copy the compiled binary
COPY --from=builder /usr/src/myapp/target/release/back /app/back

# Set runtime environment
ENV LD_LIBRARY_PATH=/usr/local/lib/libtorch:/usr/local/lib:/usr/lib
ENV DATABASE_URL=postgres://psqlx:psqlx@postgres:5432/uploads
ENV SQLX_OFFLINE=true
ENV SERVER_DATA=/server_data
ENV UTIL=/metadata

CMD ["./back"]
