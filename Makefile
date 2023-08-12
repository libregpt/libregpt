MODE := debug

trunk_args = --public-url /pkg index.html
cargo_args = --features=ssr

ifeq ($(MODE), release)
	trunk_args += --release
	cargo_args += --release
endif

build:
	trunk build $(trunk_args)
	cargo build $(cargo_args)

run: build
	./target/$(MODE)/libregpt

clean:
	trunk clean
	cargo clean

fmt:
	cargo +nightly fmt

.PHONY: build run clean fmt
