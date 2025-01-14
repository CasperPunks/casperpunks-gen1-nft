PINNED_TOOLCHAIN := $(shell cat rust-toolchain)

prepare:
	rustup target add wasm32-unknown-unknown
	rustup component add clippy --toolchain ${PINNED_TOOLCHAIN}
	rustup component add rustfmt --toolchain ${PINNED_TOOLCHAIN}

build-cep78-transfer-session:
	cd cep78-transfer-session && make build-contract

build-payment-contract:
	cd payment-contract && make build-contract

build-payment-contract-factory:
	cd payment-contract-factory && make build-contract

build-factory:
	cd factory && make build-contract

build-redeem-box:
	cd redeem-box && make build-contract

build-redeem-session:
	cd redeem-session && make build-contract

build-contracts: build-payment-contract build-factory build-payment-contract-factory build-redeem-box build-redeem-session build-cep78-transfer-session
	cd contract && cargo build --release --target wasm32-unknown-unknown
	cd client/mint_session && cargo build --release --target wasm32-unknown-unknown
	cd client/balance_of_session && cargo build --release --target wasm32-unknown-unknown
	cd client/owner_of_session && cargo build --release --target wasm32-unknown-unknown
	cd client/get_approved_session && cargo build --release --target wasm32-unknown-unknown
	cd client/is_approved_for_all_session && cargo build --release --target wasm32-unknown-unknown
	cd client/transfer_session && cargo build --release --target wasm32-unknown-unknown
	cd client/updated_receipts && cargo build --release --target wasm32-unknown-unknown
	cd test-contracts/minting_contract && cargo build --release --target wasm32-unknown-unknown
	cd test-contracts/mangle_named_keys && cargo build --release --target wasm32-unknown-unknown
	wasm-strip contract/target/wasm32-unknown-unknown/release/contract.wasm
	wasm-strip client/mint_session/target/wasm32-unknown-unknown/release/mint_call.wasm
	wasm-strip client/balance_of_session/target/wasm32-unknown-unknown/release/balance_of_call.wasm
	wasm-strip client/owner_of_session/target/wasm32-unknown-unknown/release/owner_of_call.wasm
	wasm-strip client/get_approved_session/target/wasm32-unknown-unknown/release/get_approved_call.wasm
	wasm-strip client/is_approved_for_all_session/target/wasm32-unknown-unknown/release/is_approved_for_all_call.wasm
	wasm-strip client/transfer_session/target/wasm32-unknown-unknown/release/transfer_call.wasm
	wasm-strip client/updated_receipts/target/wasm32-unknown-unknown/release/updated_receipts.wasm
	wasm-strip test-contracts/minting_contract/target/wasm32-unknown-unknown/release/minting_contract.wasm

setup-test: build-contracts build-payment-contract
	mkdir -p tests/wasm
	mkdir -p tests/wasm/1_0_0; curl -L https://github.com/casper-ecosystem/cep-78-enhanced-nft/releases/download/v1.0.0/cep-78-wasm.tar.gz | tar zxv -C tests/wasm/1_0_0/
	mkdir -p tests/wasm/1_1_0; curl -L https://github.com/casper-ecosystem/cep-78-enhanced-nft/releases/download/v1.1.0/cep-78-wasm.tar.gz | tar zxv -C tests/wasm/1_1_0/
	cp contract/target/wasm32-unknown-unknown/release/contract.wasm tests/wasm
	cp client/mint_session/target/wasm32-unknown-unknown/release/mint_call.wasm tests/wasm
	cp client/balance_of_session/target/wasm32-unknown-unknown/release/balance_of_call.wasm tests/wasm
	cp client/owner_of_session/target/wasm32-unknown-unknown/release/owner_of_call.wasm tests/wasm
	cp client/get_approved_session/target/wasm32-unknown-unknown/release/get_approved_call.wasm tests/wasm
	cp client/is_approved_for_all_session/target/wasm32-unknown-unknown/release/is_approved_for_all_call.wasm tests/wasm
	cp client/transfer_session/target/wasm32-unknown-unknown/release/transfer_call.wasm tests/wasm
	cp client/updated_receipts/target/wasm32-unknown-unknown/release/updated_receipts.wasm tests/wasm
	cp test-contracts/minting_contract/target/wasm32-unknown-unknown/release/minting_contract.wasm tests/wasm
	cp test-contracts/mangle_named_keys/target/wasm32-unknown-unknown/release/mangle_named_keys.wasm tests/wasm

test: setup-test
	cd tests && cargo test

test-one: setup-test
	cd tests && cargo test burn:: -- --nocapture

clippy:
	cd contract && cargo clippy --target wasm32-unknown-unknown --bins -- -D warnings
	cd contract && cargo clippy --no-default-features --lib -- -D warnings
	cd factory/contracts && cargo clippy --all-targets -- -D warnings
	cd payment-contract && cargo clippy --all-targets -- -D warnings
	cd payment-contract-factory && cargo clippy --all-targets -- -D warnings
	cd redeem-box && cargo clippy --all-targets -- -D warnings
	cd redeem-session && cargo clippy --all-targets -- -D warnings
	cd cep78-transfer-session && cargo clippy --all-targets -- -D warnings

check-lint: clippy
	cd contract && cargo fmt -- --check
	cd factory/contracts && cargo fmt -- --check
	cd payment-contract && cargo fmt -- --check
	cd payment-contract-factory && cargo fmt -- --check
	cd redeem-box && cargo fmt -- --check
	cd redeem-session && cargo fmt -- --check
	cd cep78-transfer-session && cargo fmt -- --check

lint: clippy
	cd contract && cargo fmt
	cd factory/contracts && cargo fmt
	cd payment-contract && cargo fmt
	cd payment-contract-factory && cargo fmt
	cd redeem-box && cargo fmt
	cd redeem-session && cargo fmt
	cd cep78-transfer-session && cargo fmt

clean:
	cd contract && cargo clean
	cd factory/contracts && cargo clean
	cd payment-contract && cargo clean
	cd payment-contract-factory && cargo clean
	cd redeem-box && cargo clean
	cd redeem-session && cargo clean
	cd cep78-transfer-session && cargo clean
	cd tests && cargo clean
	rm -rf tests/wasm
