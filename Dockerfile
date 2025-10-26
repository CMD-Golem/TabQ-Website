# Build stage
FROM rust:1.86 AS builder
RUN apt-get update
RUN apt-get install musl-tools -y

WORKDIR /app
COPY . .

RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --release --target x86_64-unknown-linux-musl

# Final stage
FROM busybox:musl
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/backend /app
COPY --from=builder /app/static/ /static/

EXPOSE 3000
ENTRYPOINT ["/app"]