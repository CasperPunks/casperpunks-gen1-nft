prepare:
	rustup target add wasm32-unknown-unknown

build-contract:
	cargo build --release -p redeem-box --target wasm32-unknown-unknown
	wasm-strip target/wasm32-unknown-unknown/release/redeem-box.wasm

build-redeem-session: 
	cd ../redeem-session && make build-contract
	cp ../redeem-session/target/wasm32-unknown-unknown/release/redeem_session.wasm tests/wasm

test: build-contract build-redeem-session
	mkdir -p tests/wasm
	cp target/wasm32-unknown-unknown/release/redeem-box.wasm tests/wasm
	cd tests && cargo test -- --nocapture

clean:
	cargo clean
	rm -rf tests/wasm/*.wasm