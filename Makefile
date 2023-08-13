MODE := debug
TLS := false

trunk_args = --public-url /pkg index.html
cargo_args = --features=ssr
run_args =

ifeq ($(MODE), release)
	trunk_args += --release
	cargo_args += --release
endif

ifeq ($(TLS), true)
	run_args += --tls
endif

build:
	trunk build $(trunk_args)
	cargo build $(cargo_args)

run: build
	./target/$(MODE)/libregpt $(run_args)

clean:
	trunk clean
	cargo clean

fmt:
	cargo +nightly fmt

.PHONY: build run clean fmt
