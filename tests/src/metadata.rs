use casper_engine_test_support::{
    ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_ACCOUNT_ADDR,
    DEFAULT_RUN_GENESIS_REQUEST,
};
use casper_types::{account::AccountHash, runtime_args, ContractHash, Key, RuntimeArgs};
use contract::constants::{
    ARG_COLLECTION_NAME, ARG_CONTRACT_WHITELIST, ARG_TOKEN_HASH, ARG_TOKEN_ID, ARG_TOKEN_META_DATA,
    ARG_TOKEN_OWNER, ENTRY_POINT_METADATA, ENTRY_POINT_MINT, ENTRY_POINT_SET_TOKEN_METADATA,
    METADATA_CEP78, TOKEN_OWNERS,
};

use crate::utility::{
    constants::{
        ARG_NFT_CONTRACT_HASH, ARG_REVERSE_LOOKUP, MINTING_CONTRACT_WASM,
        MINT_SESSION_WASM, NFT_CONTRACT_WASM, NFT_TEST_COLLECTION, TEST_PRETTY_721_META_DATA,
        TEST_PRETTY_UPDATED_721_META_DATA,
    },
    installer_request_builder::{
        InstallerRequestBuilder, MetadataMutability, MintingMode, NFTIdentifierMode,
        NFTMetadataKind, OwnerReverseLookupMode, OwnershipMode, WhitelistMode,
        TEST_CUSTOM_METADATA,
    },
    support::{
        self, assert_expected_error, get_minting_contract_hash, get_minting_contract_package,
        get_nft_contract_hash, query_stored_value,
    },
};

#[test]
fn should_prevent_update_in_immutable_mode() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(10u64)
        .with_nft_metadata_kind(NFTMetadataKind::NFT721)
        .with_identifier_mode(NFTIdentifierMode::Hash)
        .with_metadata_mutability(MetadataMutability::Immutable)
        .with_ownership_mode(OwnershipMode::Transferable)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_key: Key = support::get_nft_contract_hash(&builder).into();

    let mint_token_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA,
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_token_request).expect_success().commit();

    let token_hash: String =
        base16::encode_lower(&support::create_blake2b_hash(TEST_PRETTY_721_META_DATA));

    let update_token_metadata_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        support::get_nft_contract_hash(&builder),
        ENTRY_POINT_SET_TOKEN_METADATA,
        runtime_args! {
            ARG_TOKEN_HASH => token_hash,
            ARG_TOKEN_META_DATA => TEST_PRETTY_UPDATED_721_META_DATA
        },
    )
    .build();

    builder.exec(update_token_metadata_request).expect_failure();

    let error = builder.get_error().expect("must have error");

    support::assert_expected_error(error, 104, "must match ForbiddenMetadataUpdate(104)")
}

#[test]
fn should_prevent_install_with_hash_identifier_in_mutable_mode() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(10u64)
        .with_nft_metadata_kind(NFTMetadataKind::NFT721)
        .with_identifier_mode(NFTIdentifierMode::Hash)
        .with_metadata_mutability(MetadataMutability::Mutable)
        .build();

    builder.exec(install_request).expect_failure();

    let error = builder.get_error().expect("must fail at installation");

    assert_expected_error(error, 102, "Should raise InvalidMetadataMutability(102)")
}

#[test]
fn should_prevent_metadata_update_by_non_owner_key() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(10u64)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_identifier_mode(NFTIdentifierMode::Ordinal)
        .with_metadata_mutability(MetadataMutability::Mutable)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_key: Key = support::get_nft_contract_hash(&builder).into();

    let nft_owner_account_key = Key::Account(AccountHash::new([4u8; 32]));

    let mint_token_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINT_SESSION_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => nft_contract_key,
            ARG_TOKEN_OWNER => nft_owner_account_key,
            ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA,
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string()
        },
    )
    .build();

    builder.exec(mint_token_request).expect_success().commit();

    let _original_metadata = support::get_dictionary_value_from_key::<String>(
        &builder,
        &nft_contract_key,
        METADATA_CEP78,
        &1u64.to_string(),
    );

    let token_owner_key = support::get_dictionary_value_from_key::<Key>(
        &builder,
        &nft_contract_key,
        TOKEN_OWNERS,
        &1u64.to_string(),
    );

    assert_eq!(token_owner_key, nft_owner_account_key);

    let update_token_metadata_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        support::get_nft_contract_hash(&builder),
        ENTRY_POINT_SET_TOKEN_METADATA,
        runtime_args! {
            ARG_TOKEN_ID => 1u64,
            ARG_TOKEN_META_DATA => TEST_PRETTY_UPDATED_721_META_DATA
        },
    )
    .build();

    builder.exec(update_token_metadata_request).expect_failure();

    let error = builder.get_error().expect("must have error");

    support::assert_expected_error(error, 6, "must match InvalidTokenOwner(6)")
}

#[test]
fn should_get_metadata_using_token_id() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let minting_contract_install_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MINTING_CONTRACT_WASM,
        runtime_args! {},
    )
    .build();

    builder
        .exec(minting_contract_install_request)
        .expect_success()
        .commit();

    let minting_contract_hash = get_minting_contract_hash(&builder);
    let minting_contract_key: Key = minting_contract_hash.into();
    let minting_package_hash = get_minting_contract_package(&builder);
    println!("minting_package_hash {:?}", minting_package_hash);
    let contract_whitelist = vec![minting_contract_hash];

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, NFT_CONTRACT_WASM)
        .with_total_token_supply(100u64)
        .with_whitelist_mode(WhitelistMode::Locked)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_minting_mode(MintingMode::Installer)
        .with_reporting_mode(OwnerReverseLookupMode::Complete)
        .with_contract_whitelist(contract_whitelist.clone())
        .with_contract_minter(minting_package_hash)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_key: Key = get_nft_contract_hash(&builder).into();

    let actual_contract_whitelist: Vec<ContractHash> = query_stored_value(
        &builder,
        nft_contract_key,
        vec![ARG_CONTRACT_WHITELIST.to_string()],
    );

    assert_eq!(actual_contract_whitelist, contract_whitelist);

    let mint_runtime_args = runtime_args! {
        ARG_NFT_CONTRACT_HASH => nft_contract_key,
        ARG_TOKEN_OWNER => minting_contract_key,
        ARG_TOKEN_META_DATA => TEST_PRETTY_721_META_DATA.to_string(),
        ARG_REVERSE_LOOKUP => true,
        "count" => 1u64,
    };

    let minting_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        minting_contract_hash,
        ENTRY_POINT_MINT,
        mint_runtime_args,
    )
    .build();

    builder.exec(minting_request).expect_success().commit();

    let token_id = 1u64.to_string();
    let _minted_metadata: String = support::get_dictionary_value_from_key(
        &builder,
        &nft_contract_key,
        METADATA_CEP78,
        &token_id,
    );

    let get_metadata_request = ExecuteRequestBuilder::contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        minting_contract_hash,
        ENTRY_POINT_METADATA,
        runtime_args! {
            ARG_TOKEN_ID => 1u64,
            ARG_NFT_CONTRACT_HASH => nft_contract_key
        },
    )
    .build();

    builder.exec(get_metadata_request).expect_success().commit();
}

#[test]
fn get_schema() {
    println!(
        "{}",
        serde_json::to_string_pretty(&*TEST_CUSTOM_METADATA).unwrap()
    )
}
