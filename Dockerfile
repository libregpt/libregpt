FROM rust:1.66-slim-buster AS builder

RUN apt-get update && \
    apt-get install -y ca-certificates clang cmake
RUN update-ca-certificates

WORKDIR /tmp/libregpt

RUN cargo init

COPY Cargo.lock Cargo.toml ./

RUN cargo build --release
RUN rm -f target/release/deps/libregpt*

COPY ./src ./src
COPY ./static ./static

RUN cargo build --release

FROM debian:buster-slim

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=builder /tmp/libregpt/target/release/libregpt .

EXPOSE 80
CMD ["./libregpt"]
