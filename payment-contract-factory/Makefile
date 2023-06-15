prepare:
	rustup target add wasm32-unknown-unknown

build-contract:
	cargo build --release -p payment-contract --target wasm32-unknown-unknown
	wasm-strip target/wasm32-unknown-unknown/release/payment_contract.wasm

clean:
	cargo clean
	rm -rf tests/wasm/*.wasm