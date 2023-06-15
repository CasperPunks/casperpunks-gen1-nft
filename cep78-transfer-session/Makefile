prepare:
	rustup target add wasm32-unknown-unknown

build-contract:
	cargo build --release -p cep78-transfer-session --target wasm32-unknown-unknown
	wasm-strip target/wasm32-unknown-unknown/release/cep78_transfer_session.wasm

clean:
	cargo clean
	rm -rf tests/wasm/*.wasm