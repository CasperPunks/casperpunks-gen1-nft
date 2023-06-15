#![no_main]
#![no_std]
#![feature(type_ascription)]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

extern crate alloc;
use crate::alloc::string::ToString;
mod address;
pub mod constants;
mod entry_points;
mod error;
pub mod events;
mod helpers;
pub mod named_keys;

use crate::{constants::*, error::Error, helpers::*};
use alloc::{string::String, vec::*};
use casper_contract::{
    contract_api::{runtime, storage },
    unwrap_or_revert::UnwrapOrRevert
};
use casper_types::{
    contracts::NamedKeys, runtime_args, ContractPackageHash, HashAddr, Key, RuntimeArgs, PublicKey, AsymmetricType
};

#[no_mangle]
pub extern "C" fn init() {
    if get_key::<Key>(CONTRACT_HASH_KEY_NAME).is_some() {
        runtime::revert(Error::ContractAlreadyInitialized);
    }
    let contract_hash: Key = runtime::get_named_arg("contract_hash");
    let box_package_hash: Key = runtime::get_named_arg("box_package_hash");
    let punk_gen1_package_hash: Key = runtime::get_named_arg("punk_gen1_package_hash");
    let contract_owner: Key = runtime::get_named_arg("contract_owner");

    set_key(CONTRACT_HASH_KEY_NAME, contract_hash);
    set_key("box_package_hash", box_package_hash);
    set_key(CONTRACT_OWNER_KEY_NAME, contract_owner);
    set_key("punk_gen1_package_hash", punk_gen1_package_hash);
}

#[no_mangle]
fn call() {
    let contract_name: String = runtime::get_named_arg("contract_name");
    let punk_gen1_package_hash: Key = runtime::get_named_arg("punk_gen1_package_hash");

    let contract_owner: Key = helpers::get_named_arg_with_user_errors(
        "contract_owner",
        Error::MissingContractOwner,
        Error::InvalidContractOwner,
    )
    .unwrap_or_revert();
    let box_package_hash: Key = helpers::get_named_arg_with_user_errors(
        "box_package_hash",
        Error::MissingFeeReceiver,
        Error::InvalidFeeReceiver,
    )
    .unwrap_or_revert();

    let (contract_package_hash, _) = storage::create_contract_package_at_hash();
    let named_keys: NamedKeys = named_keys::default(
        contract_name.clone(),
        contract_owner,
        contract_package_hash
    );

    // Add new version to the package.
    let (contract_hash, _) =
        storage::add_contract_version(contract_package_hash, entry_points::default(), named_keys);

    set_key(&(contract_name.clone().to_string() + "-contract-hash"), Key::from(contract_hash));
    set_key(&(contract_name.clone().to_string() + "-contract-package-hash"), Key::from(contract_package_hash));

    runtime::call_contract::<()>(
        contract_hash,
        INIT_ENTRY_POINT_NAME,
        runtime_args! {
            "contract_hash" => Key::from(contract_hash),
            "box_package_hash" => box_package_hash,
            "contract_owner" => contract_owner,
            "punk_gen1_package_hash" => punk_gen1_package_hash
        },
    );
}

// mint function of factory
#[no_mangle]
pub extern "C" fn redeem() {
    let token_ids: Vec<u64> = helpers::get_named_arg_with_user_errors(
        "token_ids",
        Error::MissingTokenMetaData,
        Error::InvalidTokenMetaData,
    )
    .unwrap_or_revert();

    let caller = helpers::get_immediate_caller_key();

    // burn it
    let burner_pubkey = PublicKey::from_hex("020311111111111111111111111111111111111111111111111111111111deadbeef").unwrap();
    let burner_account_hash = burner_pubkey.to_account_hash();
    let burner = Key::from(burner_account_hash);
    let box_package_hash: Key = get_key("box_package_hash").unwrap();
    let punk_gen1_package_hash: Key = get_key("punk_gen1_package_hash").unwrap();

    // burn the nfts
    for token_id in &token_ids {
        transfer_from_nft(box_package_hash, caller, burner, *token_id);
    }

    call_cep78_mint(&punk_gen1_package_hash, caller, token_ids.len() as u64);
}

fn transfer_from_nft(nft_package_hash: Key, from: Key, to: Key, token_id: u64) {
    let rt = runtime_args! {
        "source_key" => from,
        "target_key" => to,
        "token_id" => token_id
    };

    let _: (String, Key) = runtime::call_versioned_contract(ContractPackageHash::new(nft_package_hash.into_hash().unwrap()), None, "transfer", rt);
}

#[no_mangle]
pub extern "C" fn transfer_owner() -> Result<(), Error> {
    only_owner();
    let new_contract_owner: Key = runtime::get_named_arg(ARG_CONTRACT_OWNER);
    set_key(CONTRACT_OWNER_KEY_NAME, new_contract_owner);
    Ok(())
}

fn call_cep78_mint(nft_contract_package: &Key, owner: Key, count: u64) {
    let nft_contract_package_addr: HashAddr = nft_contract_package.into_hash().unwrap_or_revert();
    let nft_package_hash: ContractPackageHash = ContractPackageHash::new(nft_contract_package_addr);

    let _: () = runtime::call_versioned_contract(
        nft_package_hash,
        None,
        MINT_ENTRY_POINT_NAME,
        runtime_args! {
            "token_owner" => owner,
            "count" => count,
        },
    );
}

pub fn only_owner() {
    helpers::require(
        owner_internal() == helpers::get_verified_caller().unwrap_or_revert(),
        Error::OnlyOwner,
    );
}

pub fn owner_internal() -> Key {
    let owner_key: Key = helpers::get_stored_value_with_user_errors::<Key>(
        CONTRACT_OWNER_KEY_NAME,
        Error::MissingContractOwner,
        Error::InvalidContractOwner,
    );
    owner_key
}
