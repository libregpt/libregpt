FROM rust:1.71-slim-buster AS builder

RUN rustup target add wasm32-unknown-unknown
RUN cargo install --locked trunk
RUN apt-get update && apt-get install -y clang cmake

WORKDIR /tmp/libregpt

RUN cargo init

COPY Cargo.lock Cargo.toml ./

RUN cargo build --release --features=ssr
RUN rm -f target/release/deps/libregpt*

COPY ./public ./public
COPY ./src ./src
COPY index.html tailwind.config.js Makefile ./

RUN make build MODE=release

FROM debian:buster-slim

COPY --from=builder /etc/ssl/certs /etc/ssl/certs/
COPY --from=builder /usr/share/ca-certificates /usr/share/ca-certificates/
COPY --from=builder /tmp/libregpt/dist ./dist
COPY --from=builder /tmp/libregpt/target/release/libregpt .

EXPOSE 80
CMD ["./libregpt"]
