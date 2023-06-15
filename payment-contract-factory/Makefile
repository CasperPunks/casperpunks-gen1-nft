prepare:
	rustup target add wasm32-unknown-unknown

build-contract:
	cargo build --release -p payment-contract-factory --target wasm32-unknown-unknown
	wasm-strip target/wasm32-unknown-unknown/release/payment_contract_factory.wasm

clean:
	cargo clean
	rm -rf tests/wasm/*.wasm