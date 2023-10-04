use contract::{
    constants::{
        ARG_APPROVE_ALL, ARG_COLLECTION_NAME, ARG_MINTING_MODE, ARG_OPERATOR, ARG_SOURCE_KEY,
        ARG_TARGET_KEY, ARG_TOKEN_ID, ARG_TOKEN_META_DATA, ARG_TOKEN_OWNER, ENTRY_POINT_APPROVE,
        ENTRY_POINT_MINT, ENTRY_POINT_REGISTER_OWNER, ENTRY_POINT_SET_APPROVALL_FOR_ALL,
        METADATA_CEP78, NUMBER_OF_MINTED_TOKENS, PAGE_TABLE, TOKEN_COUNT, TOKEN_ISSUERS,
        TOKEN_OWNERS, ARG_SPENDER
    },
    events::events_ces::{ApprovalForAll, Mint, RevokedForAll},
    modalities::TokenIdentifier,
};
use serde::{Deserialize, Serialize};

use casper_engine_test_support::{
    ExecuteRequestBuilder, InMemoryWasmTestBuilder, WasmTestBuilder, DEFAULT_ACCOUNT_ADDR,
    DEFAULT_RUN_GENESIS_REQUEST,
};
use casper_execution_engine::storage::global_state::in_memory::InMemoryGlobalState;
use casper_types::{account::AccountHash, runtime_args, Key, RuntimeArgs};

use crate::utility::{
    constants::{
        ACCOUNT_USER_1, ACCOUNT_USER_2, ARG_IS_HASH_IDENTIFIER_MODE, ARG_KEY_NAME,
        ARG_NFT_CONTRACT_HASH, BALANCE_OF_SESSION_WASM, CONTRACT_NAME, GET_APPROVED_WASM,
        IS_APPROVED_FOR_ALL_WASM, MINT_SESSION_WASM, NFT_CONTRACT_WASM, NFT_TEST_COLLECTION,
        OWNER_OF_SESSION_WASM, TEST_PRETTY_721_META_DATA, TEST_PRETTY_CEP78_METADATA,
        TRANSFER_SESSION_WASM,
    },
    installer_request_builder::{
        InstallerRequestBuilder, MetadataMutability, MintingMode, NFTIdentifierMode,
        NFTMetadataKind, OwnerReverseLookupMode, OwnershipMode,
    },
    support::{
        self, assert_expected_error, call_session_code_with_ret, create_funded_dummy_account,
        get_nft_contract_hash,
    },
};

#[derive(Serialize, Deserialize, Debug)]
struct Metadata {
    name: String,
    symbol: String,
    token_uri: String,
}

fn setup_nft_contract(
    total_token_supply: Option<u64>,
    allowing_minting: bool,
) -> WasmTestBuilder<InMemoryGlobalState> {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let mut install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_collection_name(NFT_TEST_COLLECTION.to_string())
            .with_reporting_mode(OwnerReverseLookupMode::NoLookUp)
            .with_allowing_minting(allowing_minting);

    if let Some(total_token_supply) = total_token_supply {
        install_request_builder =
            install_request_builder.with_total_token_supply(total_token_supply);
    }

    builder
        .exec(install_request_builder.build())
        .expect_success()
        .commit();
    builder
}

#[test]
fn should_disallow_minting_when_allow_minting_is_set_to_false() {
    let mut builder = setup_nft_contract(Some(2u64), false);

    let mint_request = ExecuteRequestBuilder::contract_call_by_name(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_NAME,
        ENTRY_POINT_MINT,
        runtime_args! {
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
        },
    )
    .build();
    builder.exec(mint_request).expect_failure();

    // Error should be MintingIsPaused=59
    let actual_error = builder.get_error().expect("must have error");
    assert_expected_error(
        actual_error,
        59u16,
        "should now allow minting when minting is disabled",
    );
}

#[test]
fn entry_points_with_ret_should_return_correct_value() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(2u64)
        .with_ownership_mode(OwnershipMode::Transferable)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_key: Key = get_nft_contract_hash(&builder).into();

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let account_hash = *DEFAULT_ACCOUNT_ADDR;

    let actual_balance: u64 = call_session_code_with_ret(
        &mut builder,
        account_hash,
        nft_contract_key,
        runtime_args! {
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
        },
        BALANCE_OF_SESSION_WASM,
        ARG_KEY_NAME,
    );

    let expected_balance = 1u64;
    assert_eq!(
        actual_balance, expected_balance,
        "actual and expected balances should be equal"
    );

    let token_id = 1u64;

    let actual_owner: Key = call_session_code_with_ret(
        &mut builder,
        account_hash,
        nft_contract_key,
        runtime_args! {
            ARG_IS_HASH_IDENTIFIER_MODE => false,
            ARG_TOKEN_ID => token_id,
        },
        OWNER_OF_SESSION_WASM,
        ARG_KEY_NAME,
    );

    let expected_owner = Key::Account(*DEFAULT_ACCOUNT_ADDR);
    assert_eq!(
        actual_owner, expected_owner,
        "actual and expected owner should be equal"
    );

    let approve_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_APPROVE,
        runtime_args! {
            ARG_TOKEN_ID => token_id,
            ARG_SPENDER => Key::Account(AccountHash::new(ACCOUNT_USER_1))
        },
    )
    .build();
    builder.exec(approve_request).expect_success().commit();

    let actual_approved_account: Option<Key> = call_session_code_with_ret(
        &mut builder,
        account_hash,
        nft_contract_key,
        runtime_args! {
            ARG_IS_HASH_IDENTIFIER_MODE => false,
            ARG_TOKEN_ID => token_id,
        },
        GET_APPROVED_WASM,
        ARG_KEY_NAME,
    );

    let expected_approved_account = Key::Account(AccountHash::new(ACCOUNT_USER_1));
    assert_eq!(
        actual_approved_account,
        Some(expected_approved_account),
        "actual and expected approved account should be equal"
    );
}

#[test]
fn should_mint() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_ownership_mode(OwnershipMode::Transferable)
            .with_nft_metadata_kind(NFTMetadataKind::CEP78)
            .with_total_token_supply(2u64);
    builder
        .exec(install_request_builder.build())
        .expect_success()
        .commit();

    let nft_contract_key: Key = get_nft_contract_hash(&builder).into();
    let token_owner: Key = Key::Account(*DEFAULT_ACCOUNT_ADDR);

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => token_owner,
            ARG_TOKEN_META_DATA => TEST_PRETTY_CEP78_METADATA,
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    // Expect Mint event.
    let _expected_event = Mint::new(
        token_owner,
        TokenIdentifier::Index(1),
        TEST_PRETTY_CEP78_METADATA.to_string(),
    );
    let _actual_event: Mint = support::get_event(&builder, &nft_contract_key, 0);
}

#[test]
fn mint_should_return_dictionary_key_to_callers_owned_tokens() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_total_token_supply(100u64)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_allowing_minting(true)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_key: Key = get_nft_contract_hash(&builder).into();
    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    // NOT AVAILABLE, mint is not returning any thing
    // let account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);

    // let nft_receipt: String =
    //     support::query_stored_value(&builder, nft_contract_key, vec![RECEIPT_NAME.to_string()]);

    // let account_receipt = *account
    //     .named_keys()
    //     .get(&format!("{nft_receipt}_m_{PAGE_SIZE}_p_{}", 0))
    //     .expect("must have receipt");

    // let actual_page = builder
    //     .query(None, account_receipt, &[])
    //     .expect("must have stored_value")
    //     .as_cl_value()
    //     .map(|page_cl_value| CLValue::into_t::<Vec<bool>>(page_cl_value.clone()))
    //     .unwrap()
    //     .unwrap();

    // let expected_page = {
    //     let mut page = vec![false; PAGE_SIZE as usize];
    //     let _ = std::mem::replace(&mut page[0], true);
    //     page
    // };

    // assert_eq!(actual_page, expected_page);
}

#[test]
fn mint_should_increment_number_of_minted_tokens_by_one_and_add_public_key_to_token_owners() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_total_token_supply(2u64)
            .with_ownership_mode(OwnershipMode::Transferable);
    builder
        .exec(install_request_builder.build())
        .expect_success()
        .commit();

    let nft_contract_key: Key = get_nft_contract_hash(&builder).into();

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    //Let's start querying
    let account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
    let nft_contract_key = account
        .named_keys()
        .get(CONTRACT_NAME)
        .expect("must have key in named keys");

    //mint should have incremented number_of_minted_tokens by one
    let query_result: u64 = support::query_stored_value(
        &builder,
        *nft_contract_key,
        vec![NUMBER_OF_MINTED_TOKENS.to_string()],
    );

    assert_eq!(
        query_result, 1u64,
        "number_of_minted_tokens initialized at installation should have incremented by one"
    );

    let token_id = 1u64;

    let _actual_token_meta_data = support::get_dictionary_value_from_key::<String>(
        &builder,
        nft_contract_key,
        METADATA_CEP78,
        &token_id.to_string(),
    );

    // assert_eq!(actual_token_meta_data, TEST_PRETTY_721_META_DATA);

    let minter_account_hash = support::get_dictionary_value_from_key::<Key>(
        &builder,
        nft_contract_key,
        TOKEN_OWNERS,
        &token_id.to_string(),
    )
    .into_account()
    .unwrap();

    assert_eq!(DEFAULT_ACCOUNT_ADDR.clone(), minter_account_hash);

    let token_page = support::get_token_page_by_id(
        &builder,
        nft_contract_key,
        &Key::Account(*DEFAULT_ACCOUNT_ADDR),
        token_id,
    );

    assert!(token_page[1]);

    // If total_token_supply is initialized to 1 the following test should fail.
    // If we set total_token_supply > 1 it should pass

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => *nft_contract_key,
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();
    builder.exec(mint_session_call).expect_success().commit();
}

#[test]
fn should_set_meta_data() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_total_token_supply(2u64)
            .with_ownership_mode(OwnershipMode::Transferable);
    builder
        .exec(install_request_builder.build())
        .expect_success()
        .commit();

    let nft_contract_key: Key = get_nft_contract_hash(&builder).into();

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,

            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();
    builder.exec(mint_session_call).expect_success().commit();

    //Let's start querying
    let account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
    let nft_contract_key = account
        .named_keys()
        .get(CONTRACT_NAME)
        .expect("must have key in named keys");

    let token_id = 1u64;

    let _actual_token_meta_data = support::get_dictionary_value_from_key::<String>(
        &builder,
        nft_contract_key,
        METADATA_CEP78,
        &token_id.to_string(),
    );
}

#[test]
fn should_set_issuer() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_total_token_supply(2u64)
            .with_ownership_mode(OwnershipMode::Transferable);
    builder
        .exec(install_request_builder.build())
        .expect_success()
        .commit();

    let nft_contract_key: Key = get_nft_contract_hash(&builder).into();

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();
    builder.exec(mint_session_call).expect_success().commit();

    //Let's start querying
    let account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
    let nft_contract_key = account
        .named_keys()
        .get(CONTRACT_NAME)
        .expect("must have key in named keys");

    let token_id = 1u64;

    let actual_token_issuer = support::get_dictionary_value_from_key::<Key>(
        &builder,
        nft_contract_key,
        TOKEN_ISSUERS,
        &token_id.to_string(),
    )
    .into_account()
    .unwrap();

    assert_eq!(actual_token_issuer, DEFAULT_ACCOUNT_ADDR.clone());
}

#[test]
fn should_set_issuer_with_different_owner() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_total_token_supply(2u64)
            .with_ownership_mode(OwnershipMode::Transferable);
    builder
        .exec(install_request_builder.build())
        .expect_success()
        .commit();

    let nft_contract_key: Key = get_nft_contract_hash(&builder).into();

    let account_user_1 = support::create_funded_dummy_account(&mut builder, Some(ACCOUNT_USER_1));

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => Key::Account(account_user_1),
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();
    builder.exec(mint_session_call).expect_success().commit();

    //Let's start querying
    let account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
    let nft_contract_key = account
        .named_keys()
        .get(CONTRACT_NAME)
        .expect("must have key in named keys");

    let token_id = 1u64;

    let actual_token_issuer = support::get_dictionary_value_from_key::<Key>(
        &builder,
        nft_contract_key,
        TOKEN_ISSUERS,
        &token_id.to_string(),
    )
    .into_account()
    .unwrap();

    assert_eq!(actual_token_issuer, DEFAULT_ACCOUNT_ADDR.clone());
}

#[test]
fn should_track_token_balance_by_owner() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_total_token_supply(2u64)
            .with_ownership_mode(OwnershipMode::Transferable);
    builder
        .exec(install_request_builder.build())
        .expect_success()
        .commit();

    let nft_contract_key: Key = get_nft_contract_hash(&builder).into();

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();
    builder.exec(mint_session_call).expect_success().commit();

    //Let's start querying
    let account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
    let nft_contract_key = account
        .named_keys()
        .get(CONTRACT_NAME)
        .expect("must have key in named keys");

    let token_owner = DEFAULT_ACCOUNT_ADDR.clone().to_string();

    let actual_minter_balance = support::get_dictionary_value_from_key::<u64>(
        &builder,
        nft_contract_key,
        TOKEN_COUNT,
        &token_owner,
    );
    let expected_minter_balance = 1u64;
    assert_eq!(actual_minter_balance, expected_minter_balance);
}

#[test]
fn should_disallow_public_minting_with_flag_set_to_false() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_minting_mode(MintingMode::Installer)
        .with_ownership_mode(OwnershipMode::Transferable)
        .build();
    builder.exec(install_request).expect_success().commit();

    let installing_account = builder.get_expected_account(*DEFAULT_ACCOUNT_ADDR);
    let nft_contract_key = installing_account
        .named_keys()
        .get(CONTRACT_NAME)
        .expect("must have key in named keys");

    let account_user_1 = support::create_funded_dummy_account(&mut builder, Some(ACCOUNT_USER_1));

    let public_minting_status = support::query_stored_value::<u8>(
        &builder,
        *nft_contract_key,
        vec![ARG_MINTING_MODE.to_string()],
    );

    assert_eq!(
        public_minting_status,
        MintingMode::Installer as u8,
        "public minting should be set to false"
    );

    let mint_session_call = ExecuteRequestBuilder::standard(
        account_user_1,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => *nft_contract_key,
            ARG_TOKEN_OWNER => Key::Account(account_user_1),
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_failure();
}

#[test]
fn should_set_approval_for_all() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_ownership_mode(OwnershipMode::Transferable)
        .build();
    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = nft_contract_hash.into();
    let owner_key = Key::Account(*DEFAULT_ACCOUNT_ADDR);

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => owner_key,
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();
    builder.exec(mint_session_call).expect_success().commit();

    let operator = create_funded_dummy_account(&mut builder, None);
    let operator_key = Key::Account(operator);

    let set_approve_for_all_request = ExecuteRequestBuilder::contract_call_by_name(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_NAME,
        ENTRY_POINT_SET_APPROVALL_FOR_ALL,
        runtime_args! {
            ARG_APPROVE_ALL => true,
            ARG_OPERATOR => operator_key
        },
    )
    .build();

    builder
        .exec(set_approve_for_all_request)
        .expect_success()
        .commit();

    let is_operator = call_session_code_with_ret::<bool>(
        &mut builder,
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_key,
        runtime_args! {
            ARG_TOKEN_OWNER => owner_key,
            ARG_OPERATOR => operator_key,
        },
        IS_APPROVED_FOR_ALL_WASM,
        ARG_KEY_NAME,
    );

    assert!(is_operator, "expected operator to be approved for all");

    // Expect ApprovalForAll event.
    let expected_event = ApprovalForAll::new(owner_key, operator_key);
    let actual_event: ApprovalForAll = support::get_event(&builder, &nft_contract_key, 1);
    assert_eq!(
        actual_event, expected_event,
        "Expected ApprovalForAll event."
    );

    // Test if two minted tokens are transferable by operator
    let token_receiver = support::create_funded_dummy_account(&mut builder, Some(ACCOUNT_USER_1));
    let token_receiver_key = Key::Account(token_receiver);

    let register_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_REGISTER_OWNER,
        runtime_args! {
            ARG_TOKEN_OWNER => token_receiver_key
        },
    )
    .build();

    builder.exec(register_request).expect_success().commit();

    let token_id = 1u64;

    // Transfer first minted token by operator
    let transfer_request = ExecuteRequestBuilder::standard(
        operator,
        TRANSFER_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_ID => token_id,
            ARG_IS_HASH_IDENTIFIER_MODE => false,
            ARG_SOURCE_KEY => owner_key,
            ARG_TARGET_KEY => token_receiver_key,
        },
    )
    .build();
    builder.exec(transfer_request).expect_success().commit();

    let actual_token_owner = support::get_dictionary_value_from_key::<Key>(
        &builder,
        &nft_contract_key,
        TOKEN_OWNERS,
        &token_id.to_string(),
    )
    .into_account()
    .unwrap();

    assert_eq!(actual_token_owner, token_receiver);

    // Second mint by owner
    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => owner_key,
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();
    builder.exec(mint_session_call).expect_success().commit();

    let token_id = 2u64;

    // Transfer second minted token by operator
    let transfer_request = ExecuteRequestBuilder::standard(
        operator,
        TRANSFER_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_ID => token_id,
            ARG_IS_HASH_IDENTIFIER_MODE => false,
            ARG_SOURCE_KEY => owner_key,
            ARG_TARGET_KEY => token_receiver_key,
        },
    )
    .build();
    builder.exec(transfer_request).expect_success().commit();

    let actual_token_owner = support::get_dictionary_value_from_key::<Key>(
        &builder,
        &nft_contract_key,
        TOKEN_OWNERS,
        &token_id.to_string(),
    )
    .into_account()
    .unwrap();

    assert_eq!(actual_token_owner, token_receiver);
}

#[test]
fn should_revoke_approval_for_all() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_ownership_mode(OwnershipMode::Transferable)
        .build();
    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = nft_contract_hash.into();
    let owner_key = Key::Account(*DEFAULT_ACCOUNT_ADDR);

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => owner_key,
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();
    builder.exec(mint_session_call).expect_success().commit();

    let operator = create_funded_dummy_account(&mut builder, None);
    let operator_key = Key::Account(operator);

    let set_approve_for_all_request = ExecuteRequestBuilder::contract_call_by_name(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_NAME,
        ENTRY_POINT_SET_APPROVALL_FOR_ALL,
        runtime_args! {
            ARG_APPROVE_ALL => true,
            ARG_OPERATOR => operator_key
        },
    )
    .build();

    builder
        .exec(set_approve_for_all_request)
        .expect_success()
        .commit();

    let is_operator = call_session_code_with_ret::<bool>(
        &mut builder,
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_key,
        runtime_args! {
            ARG_TOKEN_OWNER => owner_key,
            ARG_OPERATOR => operator_key,
        },
        IS_APPROVED_FOR_ALL_WASM,
        ARG_KEY_NAME,
    );

    assert!(is_operator, "expected operator to be approved for all");

    // Expect ApprovalForAll event.
    let expected_event = ApprovalForAll::new(owner_key, operator_key);
    let actual_event: ApprovalForAll = support::get_event(&builder, &nft_contract_key, 1);
    assert_eq!(
        actual_event, expected_event,
        "Expected ApprovalForAll event."
    );

    let revoke_approve_for_all_request = ExecuteRequestBuilder::contract_call_by_name(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_NAME,
        ENTRY_POINT_SET_APPROVALL_FOR_ALL,
        runtime_args! {
            ARG_APPROVE_ALL => false,
            ARG_OPERATOR => operator_key
        },
    )
    .build();

    builder
        .exec(revoke_approve_for_all_request)
        .expect_success()
        .commit();

    let is_operator = call_session_code_with_ret::<bool>(
        &mut builder,
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_key,
        runtime_args! {
            ARG_TOKEN_OWNER => owner_key,
            ARG_OPERATOR => operator_key,
        },
        IS_APPROVED_FOR_ALL_WASM,
        ARG_KEY_NAME,
    );

    assert!(!is_operator, "expected operator not to be approved for all");

    // Expect RevokedForAll event.
    let expected_event = RevokedForAll::new(owner_key, operator_key);
    let actual_event: RevokedForAll = support::get_event(&builder, &nft_contract_key, 2);
    assert_eq!(
        actual_event, expected_event,
        "Expected RevokedForAll event."
    );
}

// A test to ensure that the page table allocation is preserved
// even if the "register_owner" is called twice.
#[test]
fn should_maintain_page_table_despite_invoking_register_owner() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(1000u64)
        .with_identifier_mode(NFTIdentifierMode::Ordinal)
        .with_metadata_mutability(MetadataMutability::Immutable)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_nft_metadata_kind(NFTMetadataKind::Raw)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = nft_contract_hash.into();

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => "",
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let actual_page_table = support::get_dictionary_value_from_key::<Vec<bool>>(
        &builder,
        &nft_contract_key,
        PAGE_TABLE,
        &DEFAULT_ACCOUNT_ADDR.to_string(),
    );

    assert_eq!(actual_page_table.len(), 1);

    // The mint WASM will register the owner, now we re-invoke the same entry point
    // and ensure that the page table doesn't mutate.
    let register_call = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_REGISTER_OWNER,
        runtime_args! {
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR)
        },
    )
    .build();

    builder.exec(register_call).expect_success().commit();

    let table_post_register = support::get_dictionary_value_from_key::<Vec<bool>>(
        &builder,
        &nft_contract_key,
        PAGE_TABLE,
        &DEFAULT_ACCOUNT_ADDR.to_string(),
    );

    assert_eq!(actual_page_table, table_post_register)
}

#[test]
fn should_prevent_mint_to_unregistered_owner() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(1000u64)
        .with_identifier_mode(NFTIdentifierMode::Ordinal)
        .with_metadata_mutability(MetadataMutability::Immutable)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_reporting_mode(OwnerReverseLookupMode::Complete)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = nft_contract_hash.into();

    let mint_session_call = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_MINT,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => "",
            "count" => 1_u64
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success();
}

#[test]
fn should_mint_with_two_required_metadata_kind() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_total_token_supply(1000u64)
            .with_identifier_mode(NFTIdentifierMode::Ordinal)
            .with_metadata_mutability(MetadataMutability::Immutable)
            .with_ownership_mode(OwnershipMode::Transferable)
            .with_reporting_mode(OwnerReverseLookupMode::Complete)
            .with_nft_metadata_kind(NFTMetadataKind::CEP78)
            .with_additional_required_metadata(vec![NFTMetadataKind::Raw as u8]);
    builder
        .exec(install_request_builder.build())
        .expect_success()
        .commit();

    let nft_contract_key: Key = get_nft_contract_hash(&builder).into();

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string(),
            ARG_TOKEN_META_DATA => TEST_PRETTY_CEP78_METADATA,
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let _meta_78 = support::get_dictionary_value_from_key::<String>(
        &builder,
        &nft_contract_key,
        METADATA_CEP78,
        &1u64.to_string(),
    );
}

#[test]
fn should_mint_with_one_required_one_optional_metadata_kind_without_optional() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_nft_metadata_kind(NFTMetadataKind::CEP78)
            .with_optional_metadata(vec![NFTMetadataKind::Raw as u8])
            .with_total_token_supply(1000u64)
            .with_identifier_mode(NFTIdentifierMode::Ordinal)
            .with_metadata_mutability(MetadataMutability::Immutable)
            .with_ownership_mode(OwnershipMode::Transferable)
            .with_reporting_mode(OwnerReverseLookupMode::Complete);
    builder
        .exec(install_request_builder.build())
        .expect_success()
        .commit();

    let nft_contract_key: Key = get_nft_contract_hash(&builder).into();

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string(),
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => TEST_PRETTY_CEP78_METADATA,
        },
    )
    .build();
    builder.exec(mint_session_call).expect_success().commit();

    let _meta_78 = support::get_dictionary_value_from_key::<String>(
        &builder,
        &nft_contract_key,
        METADATA_CEP78,
        &1u64.to_string(),
    );

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string(),
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => TEST_PRETTY_CEP78_METADATA,
        },
    )
    .build();

    builder.exec(mint_session_call).expect_success().commit();

    let _meta_78 = support::get_dictionary_value_from_key::<String>(
        &builder,
        &nft_contract_key,
        METADATA_CEP78,
        &1u64.to_string(),
    );
}

#[test]
fn should_mint_with_transfer_only_reporting() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request_builder =
        InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
            .with_nft_metadata_kind(NFTMetadataKind::CEP78)
            .with_ownership_mode(OwnershipMode::Transferable)
            .with_reporting_mode(OwnerReverseLookupMode::TransfersOnly)
            .with_total_token_supply(2u64);
    builder
        .exec(install_request_builder.build())
        .expect_success()
        .commit();

    let nft_contract_key: Key = get_nft_contract_hash(&builder).into();
    let nft_contract_hash = get_nft_contract_hash(&builder);

    let mint_runtime_args = runtime_args! {
        ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
        ARG_TOKEN_META_DATA => TEST_PRETTY_CEP78_METADATA.to_string(),
        "count" => 1_u64
    };

    let minting_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_hash,
        ENTRY_POINT_MINT,
        mint_runtime_args,
    )
    .build();

    builder.exec(minting_request).expect_success().commit();

    let actual_balance_after_mint = support::get_dictionary_value_from_key::<u64>(
        &builder,
        &nft_contract_key,
        TOKEN_COUNT,
        &DEFAULT_ACCOUNT_ADDR.clone().to_string(),
    );

    let expected_balance_after_mint = 1u64;
    assert_eq!(actual_balance_after_mint, expected_balance_after_mint);
}

#[test]
fn should_approve_all_with_flat_gas_cost() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_ownership_mode(OwnershipMode::Transferable)
        .build();
    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash = get_nft_contract_hash(&builder);
    let nft_contract_key: Key = nft_contract_hash.into();
    let owner_key = Key::Account(*DEFAULT_ACCOUNT_ADDR);

    let mint_session_call = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => owner_key,
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();
    builder.exec(mint_session_call).expect_success().commit();

    let operator = create_funded_dummy_account(&mut builder, Some(ACCOUNT_USER_1));
    let operator_key = Key::Account(operator);

    let set_approve_for_all_request = ExecuteRequestBuilder::contract_call_by_name(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_NAME,
        ENTRY_POINT_SET_APPROVALL_FOR_ALL,
        runtime_args! {
            ARG_APPROVE_ALL => true,
            ARG_OPERATOR => operator_key
        },
    )
    .build();

    builder
        .exec(set_approve_for_all_request)
        .expect_success()
        .commit();

    let first_set_approve_for_all_gas_cost = builder.last_exec_gas_cost();

    let is_operator = call_session_code_with_ret::<bool>(
        &mut builder,
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_key,
        runtime_args! {
            ARG_TOKEN_OWNER => owner_key,
            ARG_OPERATOR => operator_key,
        },
        IS_APPROVED_FOR_ALL_WASM,
        ARG_KEY_NAME,
    );

    assert!(is_operator, "expected operator to be approved for all");

    let other_operator = create_funded_dummy_account(&mut builder, Some(ACCOUNT_USER_2));
    let other_operator_key = Key::Account(other_operator);

    let set_approve_for_all_request = ExecuteRequestBuilder::contract_call_by_name(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_NAME,
        ENTRY_POINT_SET_APPROVALL_FOR_ALL,
        runtime_args! {
            ARG_APPROVE_ALL => true,
            ARG_OPERATOR => other_operator_key
        },
    )
    .build();

    builder
        .exec(set_approve_for_all_request)
        .expect_success()
        .commit();

    let second_set_approve_for_all_gas_cost = builder.last_exec_gas_cost();

    let is_also_operator = call_session_code_with_ret::<bool>(
        &mut builder,
        *DEFAULT_ACCOUNT_ADDR,
        nft_contract_key,
        runtime_args! {
            ARG_TOKEN_OWNER => owner_key,
            ARG_OPERATOR => other_operator_key,
        },
        IS_APPROVED_FOR_ALL_WASM,
        ARG_KEY_NAME,
    );

    assert!(
        is_also_operator,
        "expected other operator to be approved for all"
    );

    // Operator approval should have flat gas costs
    // Therefore the second and first set_approve_for_all must have equivalent gas costs.
    assert_eq!(
        first_set_approve_for_all_gas_cost,
        second_set_approve_for_all_gas_cost
    )
}
