use casper_engine_test_support::{
    ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_ACCOUNT_ADDR,
    DEFAULT_RUN_GENESIS_REQUEST,
};

use casper_types::{Key, runtime_args, RuntimeArgs};
use contract::{
    constants::{
        ACCESS_KEY_NAME_1_0_0, ARG_ACCESS_KEY_NAME_1_0_0, ARG_COLLECTION_NAME, ARG_EVENTS_MODE,
        ARG_HASH_KEY_NAME_1_0_0, ARG_NAMED_KEY_CONVENTION, ARG_TOKEN_META_DATA, ARG_TOKEN_OWNER, ARG_TOTAL_TOKEN_SUPPLY,
    },
    modalities::EventsMode,
};

use crate::utility::{
    constants::{
        ARG_NFT_CONTRACT_HASH,
        ARG_NFT_CONTRACT_PACKAGE_HASH, CONTRACT_1_0_0_WASM, CONTRACT_1_1_O_WASM, MANGLE_NAMED_KEYS,
        MINT_1_0_0_WASM, NFT_CONTRACT_WASM, NFT_TEST_COLLECTION,
        NFT_TEST_SYMBOL,
    },
    installer_request_builder::{
        InstallerRequestBuilder, MetadataMutability, NFTIdentifierMode, NFTMetadataKind,
        NamedKeyConventionMode, OwnershipMode,
    },
    support,
};

const OWNED_TOKENS: &str = "owned_tokens";
const MANGLED_ACCESS_KEY_NAME: &str = "mangled_access_key";
const MANGLED_HASH_KEY_NAME: &str = "mangled_hash_key";

#[test]
fn should_not_be_able_to_reinvoke_migrate_entrypoint() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, CONTRACT_1_0_0_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(100u64)
        .with_ownership_mode(OwnershipMode::Minter)
        .with_identifier_mode(NFTIdentifierMode::Ordinal)
        .with_nft_metadata_kind(NFTMetadataKind::Raw)
        .build();

    builder.exec(install_request).expect_success().commit();

    let upgrade_to_1_1_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        CONTRACT_1_1_O_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_HASH => support::get_nft_contract_package_hash(&builder),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string(),
            ARG_NAMED_KEY_CONVENTION => NamedKeyConventionMode::V1_0Standard as u8,
            ARG_ACCESS_KEY_NAME_1_0_0 => ACCESS_KEY_NAME_1_0_0.to_string()
        },
    )
    .build();

    builder
        .exec(upgrade_to_1_1_request)
        .expect_success()
        .commit();

    let upgrade_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        NFT_CONTRACT_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_PACKAGE_HASH => support::get_nft_contract_package_hash(&builder),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string(),
            ARG_NAMED_KEY_CONVENTION => NamedKeyConventionMode::V1_0Standard as u8,
            ARG_ACCESS_KEY_NAME_1_0_0 => ACCESS_KEY_NAME_1_0_0.to_string(),
            ARG_EVENTS_MODE => EventsMode::CES as u8
        },
    )
    .build();

    builder.exec(upgrade_request).expect_success().commit();

    // Once the new contract version has been added to the package
    // calling the updated_receipts entrypoint should cause an error to be returned.
    let incorrect_upgrade_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        NFT_CONTRACT_WASM,
        runtime_args! {
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string(),
            ARG_NAMED_KEY_CONVENTION => NamedKeyConventionMode::V1_0Standard as u8,
            ARG_ACCESS_KEY_NAME_1_0_0 => ACCESS_KEY_NAME_1_0_0.to_string(),
            ARG_EVENTS_MODE => EventsMode::CES as u8
        },
    )
    .build();

    builder.exec(incorrect_upgrade_request).expect_failure();

    let error = builder.get_error().expect("must have error");

    support::assert_expected_error(error, 126u16, "must have previously migrated error");
}

#[test]
fn should_not_migrate_contracts_with_zero_token_issuance() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, CONTRACT_1_0_0_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(0u64)
        .with_ownership_mode(OwnershipMode::Minter)
        .with_identifier_mode(NFTIdentifierMode::Ordinal)
        .with_nft_metadata_kind(NFTMetadataKind::Raw)
        .build();

    builder.exec(install_request).expect_success().commit();

    let upgrade_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        NFT_CONTRACT_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_PACKAGE_HASH => support::get_nft_contract_package_hash(&builder),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string(),
            ARG_NAMED_KEY_CONVENTION => NamedKeyConventionMode::V1_0Standard as u8,
            ARG_ACCESS_KEY_NAME_1_0_0 => ACCESS_KEY_NAME_1_0_0.to_string()
        },
    )
    .build();
    builder.exec(upgrade_request).expect_failure();

    let error = builder.get_error().expect("must have error");

    support::assert_expected_error(error, 122u16, "cannot upgrade when issuance is 0");
}

#[test]
fn should_upgrade_with_custom_named_keys() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, CONTRACT_1_0_0_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(1000u64)
        .with_ownership_mode(OwnershipMode::Minter)
        .with_identifier_mode(NFTIdentifierMode::Ordinal)
        .with_nft_metadata_kind(NFTMetadataKind::Raw)
        .build();

    builder.exec(install_request).expect_success().commit();

    let nft_contract_hash_1_0_0 = support::get_nft_contract_hash_1_0_0(&builder);
    let nft_contract_key_1_0_0: Key = nft_contract_hash_1_0_0.into();

    let number_of_tokens_pre_migration = 3usize;

    for _ in 0..number_of_tokens_pre_migration {
        let mint_request = ExecuteRequestBuilder::standard(
            *DEFAULT_ACCOUNT_ADDR,
            MINT_1_0_0_WASM,
            runtime_args! {
                ARG_NFT_CONTRACT_HASH => nft_contract_key_1_0_0,
                ARG_TOKEN_OWNER => Key::Account(*DEFAULT_ACCOUNT_ADDR),
                ARG_TOKEN_META_DATA => "",
            },
        )
        .build();

        builder.exec(mint_request).expect_success().commit();
    }

    let previous_token_representation = support::get_dictionary_value_from_key::<Vec<u64>>(
        &builder,
        &nft_contract_key_1_0_0,
        OWNED_TOKENS,
        &DEFAULT_ACCOUNT_ADDR.clone().to_string(),
    );

    assert_eq!(previous_token_representation, vec![0, 1, 2]);

    let maybe_access_named_key = builder
        .query(None, Key::Account(*DEFAULT_ACCOUNT_ADDR), &[])
        .unwrap()
        .as_account()
        .unwrap()
        .named_keys()
        .get(ACCESS_KEY_NAME_1_0_0)
        .is_some();

    assert!(maybe_access_named_key);

    let mangle_named_keys_request =
        ExecuteRequestBuilder::standard(*DEFAULT_ACCOUNT_ADDR, MANGLE_NAMED_KEYS, runtime_args! {})
            .build();

    builder
        .exec(mangle_named_keys_request)
        .expect_success()
        .commit();

    let maybe_access_named_key = builder
        .query(None, Key::Account(*DEFAULT_ACCOUNT_ADDR), &[])
        .unwrap()
        .as_account()
        .unwrap()
        .named_keys()
        .get(ACCESS_KEY_NAME_1_0_0)
        .is_none();

    assert!(maybe_access_named_key);

    let improper_upgrade_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        NFT_CONTRACT_WASM,
        runtime_args! {
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string(),
            ARG_NAMED_KEY_CONVENTION => NamedKeyConventionMode::V1_0Standard as u8,
        },
    )
    .build();

    builder.exec(improper_upgrade_request).expect_failure();

    let proper_upgrade_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        NFT_CONTRACT_WASM,
        runtime_args! {
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string(),
            ARG_NAMED_KEY_CONVENTION => NamedKeyConventionMode::V1_0Custom as u8,
            ARG_ACCESS_KEY_NAME_1_0_0 => MANGLED_ACCESS_KEY_NAME.to_string(),
            ARG_HASH_KEY_NAME_1_0_0 => MANGLED_HASH_KEY_NAME.to_string(),
        },
    )
    .build();

    builder
        .exec(proper_upgrade_request)
        .expect_success()
        .commit();
}

#[test]
fn should_not_upgrade_with_larger_total_token_supply() {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&DEFAULT_RUN_GENESIS_REQUEST).commit();

    let install_request = InstallerRequestBuilder::new(*DEFAULT_ACCOUNT_ADDR, CONTRACT_1_0_0_WASM)
        .with_collection_name(NFT_TEST_COLLECTION.to_string())
        .with_collection_symbol(NFT_TEST_SYMBOL.to_string())
        .with_total_token_supply(100u64)
        .with_ownership_mode(OwnershipMode::Transferable)
        .with_identifier_mode(NFTIdentifierMode::Hash)
        .with_nft_metadata_kind(NFTMetadataKind::CEP78)
        .with_metadata_mutability(MetadataMutability::Immutable)
        .build();

    builder.exec(install_request).expect_success().commit();

    let upgrade_request = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        NFT_CONTRACT_WASM,
        runtime_args! {
            ARG_NFT_CONTRACT_PACKAGE_HASH => support::get_nft_contract_package_hash(&builder),
            ARG_COLLECTION_NAME => NFT_TEST_COLLECTION.to_string(),
            ARG_NAMED_KEY_CONVENTION => NamedKeyConventionMode::V1_0Standard as u8,
            ARG_ACCESS_KEY_NAME_1_0_0 => ACCESS_KEY_NAME_1_0_0.to_string(),
            ARG_TOTAL_TOKEN_SUPPLY => 1000u64
        },
    )
    .build();

    builder.exec(upgrade_request).expect_failure();
    let error = builder.get_error().expect("must have error");

    support::assert_expected_error(
        error,
        150u16,
        "cannot upgrade when new total token supply is larger than pre-migration one",
    );
}
