#![no_main]
#![no_std]
#![feature(type_ascription)]

#[cfg(not(target_arch = "wasm32"))]
compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

extern crate alloc;

mod address;
pub mod constants;
mod entry_points;
mod error;
pub mod events;
mod helpers;
pub mod named_keys;

use crate::{constants::*, error::Error, helpers::*};
use alloc::{
    string::{String, ToString},
    vec,
    vec::*,
};
use casper_contract::{
    contract_api::{
        account, runtime, storage,
        system::{self, transfer_from_purse_to_account, transfer_from_purse_to_purse},
    },
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    bytesrepr::{FromBytes, ToBytes},
    contracts::NamedKeys,
    runtime_args, CLTyped, ContractHash, ContractPackageHash, HashAddr, Key, RuntimeArgs, URef,
    U256,
};
use events::FactoryEvent;
use helpers::{get_immediate_caller_key, get_self_key};
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct RequestMint {
    nft_contract_package: Key,
    nft_owner_hash: String,
    request_id: String,
    request_index: U256,
    metadata: String,
}

pub const MINTING_START_TIME: &str = "minting_start_time";
pub const MINTING_END_TIME: &str = "minting_end_time";
pub const MINTING_PRICE: &str = "minting_price";

#[no_mangle]
pub extern "C" fn init() {
    if get_key::<Key>(CONTRACT_HASH_KEY_NAME).is_some() {
        runtime::revert(Error::ContractAlreadyInitialized);
    }
    let contract_hash: Key = runtime::get_named_arg(ARG_CONTRACT_HASH);
    let total_box: u64 = runtime::get_named_arg("total_box");
    let max_per_one: u8 = runtime::get_named_arg("max_per_one");

    set_key(CONTRACT_HASH_KEY_NAME, contract_hash);
    runtime::put_key("total_box", storage::new_uref(total_box as u64).into());
    runtime::put_key("max_per_one", storage::new_uref(max_per_one as u8).into());
    runtime::put_key("number_of_minted_box", storage::new_uref(0 as u64).into());

    storage::new_dictionary(ADDRESSES_WHITELIST)
        .unwrap_or_revert_with(Error::FailedToCreateDictionary);
    storage::new_dictionary(NFT_MINTED_NUMBER)
        .unwrap_or_revert_with(Error::FailedToCreateDictionary);
}

#[no_mangle]
fn call() {
    let contract_name: String = runtime::get_named_arg(NFT_FACTORY_CONTRACT_KEY_NAME);
    let contract_hash_key_name = String::from(contract_name.clone());
    let contract_package_hash_key_name = String::from(contract_name.clone() + "_package_hash");

    let contract_owner: Key = helpers::get_named_arg_with_user_errors(
        ARG_CONTRACT_OWNER,
        Error::MissingContractOwner,
        Error::InvalidContractOwner,
    )
    .unwrap_or_revert();
    let fee_receiver: Key = helpers::get_named_arg_with_user_errors(
        ARG_FEE_RECEIVER,
        Error::MissingFeeReceiver,
        Error::InvalidFeeReceiver,
    )
    .unwrap_or_revert();

    let mint_fee: U256 = helpers::get_named_arg_with_user_errors(
        ARG_MINT_FEE,
        Error::MissingCsprMintFee,
        Error::InvalidCsprMintFee,
    )
    .unwrap_or_revert();

    let total_box: u64 = helpers::get_named_arg_with_user_errors(
        "total_box",
        Error::MissingCsprMintFee,
        Error::InvalidCsprMintFee,
    )
    .unwrap_or_revert();

    let max_per_one: u8 = helpers::get_named_arg_with_user_errors(
        "max_per_one",
        Error::MissingCsprMintFee,
        Error::InvalidCsprMintFee,
    )
    .unwrap_or_revert();

    // //let fee_token: Key = runtime::get_named_arg(ARG_FEE_TOKEN_HASH);

    let (contract_package_hash, access_uref) = storage::create_contract_package_at_hash();
    let named_keys: NamedKeys = named_keys::default(
        contract_name,
        contract_owner,
        mint_fee,
        contract_package_hash,
        fee_receiver,
        None,
    );

    // Add new version to the package.
    let (contract_hash, _) =
        storage::add_contract_version(contract_package_hash, entry_points::default(), named_keys);

    // let (contract_hash, _version) = storage::new_contract(
    //     entry_points::default(),
    //     Some(named_keys),
    //     Some(String::from(contract_package_hash_key_name)),
    //     None,
    // );

    runtime::put_key(CONTRACT_OWNER_KEY_NAME, contract_owner);
    // runtime::put_key(DEV, dev);
    runtime::put_key(
        contract_hash_key_name.as_str(),
        Key::from(contract_package_hash),
    );
    runtime::put_key(BOX_FACTORY_ACCESS, Key::from(access_uref));

    runtime::call_contract::<()>(
        contract_hash,
        INIT_ENTRY_POINT_NAME,
        runtime_args! {
            "contract_hash" => Key::from(contract_hash),
            "total_box" => total_box,
            "max_per_one" => max_per_one,
        },
    );
}

#[no_mangle]
pub extern "C" fn set_addresses_whitelist() -> Result<(), Error> {
    // Check caller must be DEV account
    let caller = get_immediate_caller_key();
    let current_owner = helpers::get_stored_value_with_user_errors::<Key>(
        CONTRACT_OWNER_KEY_NAME,
        Error::MissingContractOwner,
        Error::InvalidContractOwner,
    );

    if caller != current_owner {
        runtime::revert(Error::InvalidContractOwner);
    }

    // Take valid new_addresses from runtime args
    let new_addresses_whitelist = helpers::get_named_arg_with_user_errors::<Vec<Key>>(
        ARG_NEW_ADDRESSES_WHITELIST,
        Error::MissingNewAddressWhitelist,
        Error::InvalidNewAddressWhitelist,
    )
    .unwrap_or_revert_with(Error::CannotGetWhitelistAddrressArg);

    let is_whitelist = helpers::get_named_arg_with_user_errors::<bool>(
        ARG_IS_WHITELIST,
        Error::MissingNumberOfTickets,
        Error::InvalidNumberOfTickets,
    )
    .unwrap_or_revert_with(Error::CannotGetNumberOfTickets);

    let mut new_addresses: Vec<Key> = Vec::new();
    // Get new address if valid.
    for new_address in new_addresses_whitelist {
        // Validate new_address is account type
        if new_address.into_account().is_none() {
            runtime::revert(Error::InputMustBeAccountHash);
        }
        let account_key = make_dictionary_item_key_for_account(new_address: Key);
        // push new_address in array new_addresses
        new_addresses.push(new_address.clone());
    }

    // Add new_addresses into dictionary

    for new_address in new_addresses {
        let account_key_1 = make_dictionary_item_key_for_account(new_address: Key);

        write_dictionary_value_from_key(ADDRESSES_WHITELIST, &account_key_1, is_whitelist);

        write_dictionary_value_from_key(NFT_MINTED_NUMBER, &account_key_1, 0 as u8);
    }
    Ok(())
}

#[no_mangle]
pub extern "C" fn update_addresses_whitelist() -> Result<(), Error> {
    let caller = get_immediate_caller_key();
    let current_owner = helpers::get_stored_value_with_user_errors::<Key>(
        CONTRACT_OWNER_KEY_NAME,
        Error::MissingContractOwner,
        Error::InvalidContractOwner,
    );

    if caller != current_owner {
        runtime::revert(Error::InvalidContractOwner);
    }

    // Take valid new_addresses from runtime args
    let new_addresses_whitelist = helpers::get_named_arg_with_user_errors::<Vec<Key>>(
        ARG_NEW_ADDRESSES_WHITELIST,
        Error::MissingNewAddressWhitelist,
        Error::InvalidNewAddressWhitelist,
    )
    .unwrap_or_revert_with(Error::CannotGetWhitelistAddrressArg);

    let number_of_tickets = helpers::get_named_arg_with_user_errors::<u8>(
        ARG_NUMBER_OF_TICKETS,
        Error::MissingNumberOfTickets,
        Error::InvalidNumberOfTickets,
    )
    .unwrap_or_revert_with(Error::CannotGetNumberOfTickets);

    let mut new_addresses: Vec<Key> = Vec::new();
    // Get new address if valid.
    for new_address in new_addresses_whitelist {
        // Validate new_address is account type
        if new_address.into_account().is_none() {
            runtime::revert(Error::InputMustBeAccountHash);
        }
        // push new_address in array new_addresses
        new_addresses.push(new_address.clone());
    }

    // Add new_addresses into dictionary

    for new_address in new_addresses {
        let account_key_1 = make_dictionary_item_key_for_account(new_address: Key);

        write_dictionary_value_from_key(ADDRESSES_WHITELIST, &account_key_1, number_of_tickets);
    }
    Ok(())
}

// mint function of factory
#[no_mangle]
pub extern "C" fn mint() {
    let nft_owner: Key = helpers::get_named_arg_with_user_errors(
        "token_owner",
        Error::MissingTokenMetaData,
        Error::InvalidTokenMetaData,
    )
    .unwrap_or_revert();
    if nft_owner.into_account().is_none() {
        runtime::revert(Error::CallerMustBeAccountHash);
    }

    let nft_contract_package: Key = helpers::get_named_arg_with_user_errors::<Key>(
        ARG_NFT_CONTRACT_PACKAGE,
        Error::MissingNftContractPackage,
        Error::InvalidNftContractPackage,
    )
    .unwrap_or_revert(); //Contract hash of NFT CASPERPUNK

    let count: u8 = helpers::get_named_arg_with_user_errors(
        "count",
        Error::MissingNftContractPackage,
        Error::InvalidNftContractPackage,
    )
    .unwrap_or_revert();

    let nft_owner_key = make_dictionary_item_key_for_account(nft_owner: Key);

    // Check if new_address already in dictionary
    let is_wl = get_dictionary_value_from_key::<bool>(ADDRESSES_WHITELIST, &nft_owner_key).unwrap();
    if is_wl != true {
        runtime::revert(Error::InvalidContext);
    }
    if get_dictionary_value_from_key::<u8>(NFT_MINTED_NUMBER, &nft_owner_key).is_none() {
        runtime::revert(Error::InvalidWhiteListAddress);
    }

    let max_per_one: u8 = helpers::get_stored_value_with_user_errors(
        "max_per_one",
        Error::InvalidContext,
        Error::InvalidContext,
    );

    let nft_minted: u8 = get_dictionary_value_from_key::<u8>(NFT_MINTED_NUMBER, &nft_owner_key)
        .unwrap_or_revert_with(Error::CanotGetNftMintedNumber);
    if nft_minted > max_per_one {
        runtime::revert(Error::ReachMaximumNumberOfMinting);
    }

    let number_of_minted_box: u64 = helpers::get_stored_value_with_user_errors(
        "number_of_minted_box",
        Error::InvalidContext,
        Error::InvalidContext,
    );

    let total_box: u64 = helpers::get_stored_value_with_user_errors(
        "total_box",
        Error::InvalidContext,
        Error::InvalidContext,
    );

    if number_of_minted_box + (count as u64) > total_box {
        runtime::revert(Error::InvalidContext);
    }
    // Transfer_from cspr from user's purse to fee_receiver

    let fee_receiver: Key = helpers::get_stored_value_with_user_errors::<Key>(
        FEE_RECEIVER,
        Error::MissingFeeReceiver,
        Error::InvalidFeeReceiver,
    );

    let wcspr_mint_fee: U256 = helpers::get_stored_value_with_user_errors::<U256>(
        MINT_FEE,
        Error::MissingCsprMintFee,
        Error::InvalidCsprMintFee,
    );

    let allowed_cspr_amount = helpers::get_named_arg_with_user_errors::<U256>(
        AMOUNT_RUNTIME_ARG_NAME,
        Error::MissingAmount,
        Error::InvalidAmount,
    )
    .unwrap_or_revert_with(Error::CannotGetAmount);
    let required_amount: U256 = wcspr_mint_fee * U256::from(count as u8);
    if allowed_cspr_amount < required_amount {
        runtime::revert(Error::NotEnoughAmount)
    }
    let src_purse: URef = helpers::get_named_arg_with_user_errors::<URef>(
        ARG_SRC_PURSE,
        Error::MissingSrcPurse,
        Error::InvalidSrcPurse,
    )
    .unwrap_or_revert_with(Error::CanNotGetUserPurse); //Contract hash of NFT CASPERPUNK

    transfer_from_purse_to_account(
        src_purse,
        fee_receiver.into_account().unwrap(),
        u256_to_u512(required_amount),
        None,
    )
    .unwrap_or_revert_with(Error::CanNotTransferCSPR);

    let token_metadata: String = helpers::get_named_arg_with_user_errors::<String>(
        ARG_TOKEN_META_DATA,
        Error::MissingTokenMetaData,
        Error::InvalidTokenMetaData,
    )
    .unwrap_or_revert();

    // TODO: CALL MINT function of CEP78
    call_cep78_mint(&nft_contract_package, nft_owner, token_metadata, count);

    write_dictionary_value_from_key(
        NFT_MINTED_NUMBER,
        &nft_owner_key,
        (nft_minted + count) as u8,
    );
    set_key(
        "number_of_minted_box",
        number_of_minted_box + (count as u64),
    );
    events::emit(&FactoryEvent::MintFactory {
        owner: nft_owner,
        minted: (nft_minted + count) as u8,
    });
}

#[no_mangle]
pub extern "C" fn transfer_owner() -> Result<(), Error> {
    let new_contract_owner: Key = runtime::get_named_arg(ARG_CONTRACT_OWNER);
    let caller = helpers::get_verified_caller().unwrap_or_revert();
    let current_contract_owner = helpers::get_stored_value_with_user_errors::<Key>(
        CONTRACT_OWNER_KEY_NAME,
        Error::MissingContractOwner,
        Error::InvalidContractOwner,
    );

    if caller != current_contract_owner {
        runtime::revert(Error::InvalidContractOwner);
    }
    set_key(CONTRACT_OWNER_KEY_NAME, new_contract_owner);
    Ok(())
}

#[no_mangle]
pub extern "C" fn change_fee_receiver() -> Result<(), Error> {
    let new_fee_receiver: Key = runtime::get_named_arg(ARG_FEE_RECEIVER);
    let caller = helpers::get_verified_caller().unwrap_or_revert();
    let current_contract_owner = helpers::get_stored_value_with_user_errors::<Key>(
        CONTRACT_OWNER_KEY_NAME,
        Error::MissingContractOwner,
        Error::InvalidContractOwner,
    );

    if caller != current_contract_owner {
        runtime::revert(Error::InvalidContractOwner);
    }
    set_key(FEE_RECEIVER, new_fee_receiver);
    Ok(())
}

#[no_mangle]
pub extern "C" fn change_wcspr_contract() -> Result<(), Error> {
    let new_wcspr_contract: Key = runtime::get_named_arg(ARG_WCSPR_CONTRACT);
    let caller = helpers::get_verified_caller().unwrap_or_revert();
    let current_contract_owner = helpers::get_stored_value_with_user_errors::<Key>(
        CONTRACT_OWNER_KEY_NAME,
        Error::MissingContractOwner,
        Error::InvalidContractOwner,
    );

    if caller != current_contract_owner {
        runtime::revert(Error::InvalidContractOwner);
    }
    set_key(WCSPR_CONTRACT, new_wcspr_contract);
    Ok(())
}

#[no_mangle]
pub extern "C" fn change_mint_fee() -> Result<(), Error> {
    let new_wcspr_mint_fee: U256 = runtime::get_named_arg(ARG_MINT_FEE);
    let caller = helpers::get_verified_caller().unwrap_or_revert();
    let current_contract_owner = helpers::get_stored_value_with_user_errors::<Key>(
        CONTRACT_OWNER_KEY_NAME,
        Error::MissingContractOwner,
        Error::InvalidContractOwner,
    );

    if caller != current_contract_owner {
        runtime::revert(Error::InvalidContractOwner);
    }
    set_key(MINT_FEE, new_wcspr_mint_fee);
    Ok(())
}
fn call_cep78_mint(nft_contract_package: &Key, owner: Key, metadata: String, count: u8) {
    let nft_contract_package_addr: HashAddr = nft_contract_package.into_hash().unwrap_or_revert();
    let nft_package_hash: ContractPackageHash = ContractPackageHash::new(nft_contract_package_addr);

    let _: () = runtime::call_versioned_contract(
        nft_package_hash,
        None,
        MINT_ENTRY_POINT_NAME,
        runtime_args! {
            "token_owners" => vec![owner],
            "token_meta_data" => metadata,
            "number_of_boxs" => vec![count],
        },
    );

    // let contract_hash_addr: HashAddr = contract_hash.into_hash().unwrap_or_revert();
    // let contract_hash: ContractHash = ContractHash::new(contract_hash_addr);
    // let _: (String, Key, String) = runtime::call_contract(
    //     contract_hash,
    //     MINT_ENTRY_POINT_NAME,
    //     runtime_args! {
    //         ARG_TOKEN_OWNER => owner,
    //         ARG_TOKEN_META_DATA => metadata,
    //     },
    // );
}

// pub fn minting_valid_time() {
//     let start_time: u64 = helpers::get_stored_value_with_user_errors(
//         MINTING_START_TIME,
//         NFTCoreError::MissingMintingStart,
//         NFTCoreError::InvalidMintingStart,
//     );
//     let end_time: u64 = helpers::get_stored_value_with_user_errors(
//         MINTING_END_TIME,
//         NFTCoreError::MissingMintingEnd,
//         NFTCoreError::InvalidMintingEnd,
//     );
//     let current_time_sec = helpers::current_block_timestamp_sec();
//     helpers::require(
//         start_time <= current_time_sec && current_time_sec <= end_time,
//         NFTCoreError::MintingTimeInvalid,
//     );
// }
