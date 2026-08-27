FROM rust:alpine AS builder

WORKDIR /use/src/builder

COPY . .

RUN cargo install --path .

FROM alpine:latest

COPY --from=builder /usr/local/cargo/bin/my-http-server /usr/local/bin/my-http-server

CMD ["my-http-server"]
