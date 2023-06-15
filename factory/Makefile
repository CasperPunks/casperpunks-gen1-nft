prepare:
	cd contracts && rustup target add wasm32-unknown-unknown

build-contract:
	cd contracts && cargo build --release --target wasm32-unknown-unknown
	wasm-strip target/wasm32-unknown-unknown/release/contract.wasm 2>/dev/null | true

test: build-contract
	mkdir -p tests/wasm
	cp contracts/target/wasm32-unknown-unknown/release/contract.wasm tests/wasm
	../payment-contract/target/wasm32-unknown-unknown/release/payment_contract.wasm tests/wasm
	cd tests && cargo test -- --nocapture

clippy:
	cd contract && cargo clippy --all-targets -- -D warnings
	cd tests && cargo clippy --all-targets -- -D warnings

check-lint: clippy
	cd contract && cargo fmt -- --check
	cd tests && cargo fmt -- --check

lint: clippy
	cd contract && cargo fmt
	cd tests && cargo fmt

clean:
	cd contract && cargo clean
	cd tests && cargo clean
	rm -rf tests/wasm
