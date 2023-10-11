FROM rust:1.71.0-slim-buster as builder
WORKDIR /usr/src/myapp
COPY . .
RUN cargo install --path ./kitchen
FROM debian:buster-slim
RUN apt-get update & apt-get install -y extra-runtime-dependencies & rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/kitchen /usr/local/bin/kitchen
CMD ["kitchen"]