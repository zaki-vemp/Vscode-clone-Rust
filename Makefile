.PHONY: build run check package deb clean

build:
	cargo build --release --locked

run:
	cargo run --release --locked -- $(ARGS)

check:
	cargo fmt --check
	cargo clippy --all-targets --locked -- -D warnings
	cargo test --locked

package:
	python3 scripts/package.py

deb:
	python3 scripts/package.py --format deb

clean:
	cargo clean
