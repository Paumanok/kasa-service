FROM rust:1-alpine3.20

WORKDIR /usr/src/myapp
COPY . .

RUN cargo install --path .

CMD ["kasa-service"]

