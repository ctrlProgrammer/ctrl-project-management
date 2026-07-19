.PHONY: test build release clippy clean

test:
	cargo test

build:
	cargo build

release:
	cargo build --release

clippy:
	cargo clippy

clean:
	cargo clean
