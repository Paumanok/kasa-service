FROM rust:1.80 as builder
WORKDIR /usr/src/myapp
COPY . .
RUN cargo install --path .

FROM rust:1.80-slim-bookworm as runner
#RUN apt-get update && apt-get install -y libc6 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/kasa-service /usr/local/bin/kasa-service
EXPOSE 4000
CMD ["kasa-service"]

