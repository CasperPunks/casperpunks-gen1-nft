use casper_engine_test_support::{
    ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_RUN_GENESIS_REQUEST,
    DEFAULT_ACCOUNT_ADDR, MINIMUM_ACCOUNT_CREATION_BALANCE,
};

use casper_types::{
    account::AccountHash, bytesrepr::FromBytes, CLTyped, runtime_args, system::mint,
    ContractHash, ContractPackageHash, Key, PublicKey, RuntimeArgs, crypto::SecretKey, U256
};
use std::convert::TryInto;
const REDEEM_BOX: &str = "redeem-box.wasm";
const REDEEM_SESSION: &str = "redeem_session.wasm";
const RESULT_KEY: &str = "result";
const TOKEN_TOTAL_SUPPLY: u128 = 1_000_000_000_000_000_000_000_000_000;
const LOOTBOX_WASM: &str = "lootbox.wasm";

fn get_account1_addr() -> AccountHash {
    let sk: SecretKey = SecretKey::secp256k1_from_bytes(&[221u8; 32]).unwrap();
    let pk: PublicKey = PublicKey::from(&sk);
    let a: AccountHash = pk.to_account_hash();
    a
}

fn get_account2_addr() -> AccountHash {
    let sk: SecretKey = SecretKey::secp256k1_from_bytes(&[212u8; 32]).unwrap();
    let pk: PublicKey = PublicKey::from(&sk);
    let a: AccountHash = pk.to_account_hash();
    a
}

#[derive(Copy, Clone)]
struct TestContext {
    redeem_box: Key
}

fn exec_call(builder: &mut InMemoryWasmTestBuilder, account_hash: AccountHash, contract_pacakge_hash: Key, fun_name: &str, args: RuntimeArgs, expect_success: bool) {
    let request = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        account_hash,
        ContractPackageHash::new(contract_pacakge_hash.into_hash().unwrap()),
        None,
        fun_name,
        args
    ).build();
    if expect_success {
        builder.exec(request).expect_success().commit();
    } else {
        builder.exec(request).expect_failure();
    }
}

fn setup() -> (InMemoryWasmTestBuilder, TestContext) {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&*DEFAULT_RUN_GENESIS_REQUEST);

    let id: Option<u64> = None;
    let transfer_1_args = runtime_args! {
        mint::ARG_TARGET => get_account1_addr(),
        mint::ARG_AMOUNT => MINIMUM_ACCOUNT_CREATION_BALANCE,
        mint::ARG_ID => id,
    };
    let transfer_2_args = runtime_args! {
        mint::ARG_TARGET => get_account2_addr(),
        mint::ARG_AMOUNT => MINIMUM_ACCOUNT_CREATION_BALANCE,
        mint::ARG_ID => id,
    };

    let transfer_request_1 =
        ExecuteRequestBuilder::transfer(*DEFAULT_ACCOUNT_ADDR, transfer_1_args).build();
    let transfer_request_2 =
        ExecuteRequestBuilder::transfer(*DEFAULT_ACCOUNT_ADDR, transfer_2_args).build();

        let deploy_nft = ExecuteRequestBuilder::standard(
            *DEFAULT_ACCOUNT_ADDR,
            LOOTBOX_WASM,
            runtime_args! {
                "named_key_convention" => 0u8,
                "collection_name" => "Punk Lootbox".to_string(),
                "collection_symbol" => "PLB".to_string(),
                "total_token_supply" => 10000u64,
                "allow_minting" => true,
                "minting_mode" => 1u8,
                "ownership_mode" => 2u8,
                "nft_kind" => 1u8,
                "holder_mode" => 2u8,
                "whitelist_mode" => 0u8,
                "contract_whitelist" => Vec::<ContractHash>::new(),
                "nft_metadata_kind" => 0u8,
                "additional_required_metadata" => bytesrepr::Bytes::new(),
                "optional_metadata" => bytesrepr::Bytes::new(),
                "json_schema" => "schema".to_string(),
                "identifier_mode" => 0u8,
                "metadata_mutability" => 0u8,
                "burn_mode" => 0u8,
                "owner_reverse_lookup_mode" => 1u8,
                "the_contract_owner" => Key::from(*DEFAULT_ACCOUNT_ADDR),
                "minting_start_time" => 0u64,
                "minting_end_time" => 999999999999u64,
                "minting_price" => U256::from(1_000_000_000u128),
                "cspr_receiver" => Key::from(*DEFAULT_ACCOUNT_ADDR)
            },
        )
        .build();

    let install_request_1 = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        REDEEM_SESSION,
        runtime_args! {
            "contract_name" => "erc20_token_factory".to_string()
        },
    )
    .build();


    builder.exec(transfer_request_1).expect_success().commit();
    builder.exec(transfer_request_2).expect_success().commit();
    builder.exec(install_request_1).expect_success().commit();
    builder.exec(deploy_nft).expect_success().commit();

    let account = builder
        .get_account(*DEFAULT_ACCOUNT_ADDR)
        .expect("should have account");

    let token_factory = account
        .named_keys()
        .get(&"erc20_token_factory_package_hash".to_string())
        .and_then(|key| key.into_hash())
        .map(ContractPackageHash::new)
        .expect("should have contract hash");

    let token_factory = Key::from(token_factory);

    let tc = TestContext {
        token_factory
    };

    (builder, tc)
}

#[test]
fn test_request_bridge() {
    setup();
    println!("gas request_bridge_erc20 {:?}", builder.last_exec_gas_cost());
}

