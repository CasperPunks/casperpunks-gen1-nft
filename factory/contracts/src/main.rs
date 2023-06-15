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
use alloc::{string::String, vec};
use casper_contract::{
    contract_api::{runtime, storage, system::transfer_from_purse_to_account},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    contracts::NamedKeys, runtime_args, ContractPackageHash, HashAddr, Key, RuntimeArgs, URef, U256,
};
use events::FactoryEvent;

pub const MINTING_START_TIME: &str = "minting_start_time";
pub const MINTING_START_TIME_R3: &str = "minting_start_time_r3";
pub const MINTING_END_TIME: &str = "minting_end_time";

#[no_mangle]
pub extern "C" fn init() {
    if get_key::<Key>(CONTRACT_HASH_KEY_NAME).is_some() {
        runtime::revert(Error::ContractAlreadyInitialized);
    }
    let contract_hash: Key = runtime::get_named_arg(ARG_CONTRACT_HASH);
    let total_box: u64 = runtime::get_named_arg("total_box");
    let total_box_r3: u64 = runtime::get_named_arg("total_box_r3");
    let max_per_one: u8 = runtime::get_named_arg("max_per_one");
    let max_per_one_r3: u8 = runtime::get_named_arg("max_per_one_r3");
    let start_time: u64 = runtime::get_named_arg(MINTING_START_TIME);
    let start_time_r3: u64 = runtime::get_named_arg(MINTING_START_TIME_R3);
    let end_time: u64 = runtime::get_named_arg(MINTING_END_TIME);
    let nft_contract_package: Key = runtime::get_named_arg(ARG_NFT_CONTRACT_PACKAGE);

    set_key(CONTRACT_HASH_KEY_NAME, contract_hash);
    set_key(ARG_NFT_CONTRACT_PACKAGE, nft_contract_package);
    runtime::put_key("total_box", storage::new_uref(total_box as u64).into());
    runtime::put_key(
        "total_box_r3",
        storage::new_uref(total_box_r3 as u64).into(),
    );

    runtime::put_key("max_per_one", storage::new_uref(max_per_one as u8).into());
    runtime::put_key(
        "max_per_one_r3",
        storage::new_uref(max_per_one_r3 as u8).into(),
    );
    runtime::put_key("number_of_minted_box", storage::new_uref(0_u64).into());
    runtime::put_key("number_of_minted_box_r3", storage::new_uref(0_u64).into());
    runtime::put_key(
        MINTING_START_TIME,
        storage::new_uref(start_time as u64).into(),
    );
    runtime::put_key(
        MINTING_START_TIME_R3,
        storage::new_uref(start_time_r3 as u64).into(),
    );
    runtime::put_key(MINTING_END_TIME, storage::new_uref(end_time as u64).into());

    storage::new_dictionary(ADDRESSES_WHITELIST)
        .unwrap_or_revert_with(Error::FailedToCreateDictionary);
    storage::new_dictionary(NFT_MINTED_NUMBER)
        .unwrap_or_revert_with(Error::FailedToCreateDictionary);
    storage::new_dictionary(NFT_MINTED_NUMBER_R3)
        .unwrap_or_revert_with(Error::FailedToCreateDictionary);
}

#[no_mangle]
fn call() {
    let contract_name: String = runtime::get_named_arg(NFT_FACTORY_CONTRACT_KEY_NAME);
    let contract_hash_key_name = contract_name.clone();

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
    let start_time: u64 = helpers::get_named_arg_with_user_errors(
        MINTING_START_TIME,
        Error::MissingMintingStart,
        Error::InvalidMintingStart,
    )
    .unwrap_or_revert();
    let start_time_r3: u64 = helpers::get_named_arg_with_user_errors(
        MINTING_START_TIME_R3,
        Error::MissingMintingStart,
        Error::InvalidMintingStart,
    )
    .unwrap_or_revert();

    let end_time: u64 = helpers::get_named_arg_with_user_errors(
        MINTING_END_TIME,
        Error::MissingMintingEnd,
        Error::InvalidMintingEnd,
    )
    .unwrap_or_revert();

    let mint_fee: U256 = helpers::get_named_arg_with_user_errors(
        ARG_MINT_FEE,
        Error::MissingCsprMintFee,
        Error::InvalidCsprMintFee,
    )
    .unwrap_or_revert();

    let mint_fee_r3: U256 = helpers::get_named_arg_with_user_errors(
        ARG_MINT_FEE_R3,
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

    let total_box_r3: u64 = helpers::get_named_arg_with_user_errors(
        "total_box_r3",
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

    let max_per_one_r3: u8 = helpers::get_named_arg_with_user_errors(
        "max_per_one_r3",
        Error::MissingCsprMintFee,
        Error::InvalidCsprMintFee,
    )
    .unwrap_or_revert();

    let nft_contract_package: Key = runtime::get_named_arg(ARG_NFT_CONTRACT_PACKAGE);

    let (contract_package_hash, access_uref) = storage::create_contract_package_at_hash();
    let named_keys: NamedKeys = named_keys::default(
        contract_name,
        contract_owner,
        mint_fee,
        mint_fee_r3,
        contract_package_hash,
        fee_receiver,
        None,
    );

    // Add new version to the package.
    let (contract_hash, _) =
        storage::add_contract_version(contract_package_hash, entry_points::default(), named_keys);

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
            "total_box_r3" => total_box_r3,
            "max_per_one" => max_per_one,
            "max_per_one_r3" => max_per_one_r3,
            "minting_start_time" => start_time,
            "minting_start_time_r3" => start_time_r3,
            "minting_end_time" => end_time,
            ARG_NFT_CONTRACT_PACKAGE => nft_contract_package
        },
    );
}

#[no_mangle]
pub extern "C" fn set_addresses_whitelist() {
    // Check caller must be DEV account
    only_owner();
}

// mint function of factory
#[no_mangle]
pub extern "C" fn mint() {
    minting_valid_time();
    let nft_owner: Key = helpers::get_named_arg_with_user_errors(
        "token_owner",
        Error::MissingTokenMetaData,
        Error::InvalidTokenMetaData,
    )
    .unwrap_or_revert();
    if nft_owner.into_account().is_none() {
        runtime::revert(Error::CallerMustBeAccountHash);
    }

    let count: u8 = helpers::get_named_arg_with_user_errors(
        "count",
        Error::MissingNftContractPackage,
        Error::InvalidNftContractPackage,
    )
    .unwrap_or_revert();

    let nft_owner_key = make_dictionary_item_key_for_account(nft_owner: Key);

    let is_round2_finished = is_round2_done();

    let max_per_one_key = if is_round2_finished {
        "max_per_one_r3"
    } else {
        "max_per_one"
    };

    let max_per_one: u8 = helpers::get_stored_value_with_user_errors(
        max_per_one_key,
        Error::InvalidContext,
        Error::InvalidContext,
    );

    let nft_minted_number_key = if is_round2_finished {
        NFT_MINTED_NUMBER_R3
    } else {
        NFT_MINTED_NUMBER
    };

    let nft_minted =
        match get_dictionary_value_from_key::<u8>(nft_minted_number_key, &nft_owner_key) {
            Some(minted) => minted as u8,
            None => 0u8,
        };
    if nft_minted + count > max_per_one {
        runtime::revert(Error::ReachMaximumNumberOfMinting);
    }

    let number_of_minted_box_key = if is_round2_finished {
        "number_of_minted_box_r3"
    } else {
        "number_of_minted_box"
    };
    let number_of_minted_box: u64 = helpers::get_stored_value_with_user_errors(
        number_of_minted_box_key,
        Error::InvalidContext,
        Error::InvalidContext,
    );

    let total_box_key = if is_round2_finished {
        "total_box_r3"
    } else {
        "total_box"
    };

    let total_box: u64 = helpers::get_stored_value_with_user_errors(
        total_box_key,
        Error::InvalidContext,
        Error::InvalidContext,
    );

    if number_of_minted_box + (count as u64) > total_box {
        runtime::revert(Error::InvalidContext);
    }

    let fee_receiver: Key = helpers::get_stored_value_with_user_errors::<Key>(
        FEE_RECEIVER,
        Error::MissingFeeReceiver,
        Error::InvalidFeeReceiver,
    );

    let mint_fee_key = if is_round2_finished {
        MINT_FEE_R3
    } else {
        MINT_FEE
    };

    let wcspr_mint_fee: U256 = helpers::get_stored_value_with_user_errors::<U256>(
        mint_fee_key,
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

    let nft_contract_package: Key = helpers::get_key(ARG_NFT_CONTRACT_PACKAGE).unwrap();

    call_cep78_mint(&nft_contract_package, nft_owner, count);

    write_dictionary_value_from_key(
        nft_minted_number_key,
        &nft_owner_key,
        (nft_minted + count) as u8,
    );

    set_key(
        number_of_minted_box_key,
        number_of_minted_box + (count as u64),
    );
    events::emit(&FactoryEvent::MintFactory {
        owner: nft_owner,
        minted: (nft_minted + count) as u8,
    });
}

#[no_mangle]
pub extern "C" fn transfer_owner() {
    only_owner();
    let new_contract_owner: Key = runtime::get_named_arg(ARG_CONTRACT_OWNER);
    set_key(CONTRACT_OWNER_KEY_NAME, new_contract_owner);
}

#[no_mangle]
pub extern "C" fn change_fee_receiver() {
    only_owner();
    let new_fee_receiver: Key = runtime::get_named_arg(ARG_FEE_RECEIVER);
    set_key(FEE_RECEIVER, new_fee_receiver);
}

#[no_mangle]
pub extern "C" fn change_mint_fee() {
    only_owner();
    let new_wcspr_mint_fee: U256 = runtime::get_named_arg(ARG_MINT_FEE);
    set_key(MINT_FEE, new_wcspr_mint_fee);
}

#[no_mangle]
pub extern "C" fn change_mint_fee_r3() {
    only_owner();
    let new_wcspr_mint_fee: U256 = runtime::get_named_arg(ARG_MINT_FEE_R3);
    set_key(MINT_FEE_R3, new_wcspr_mint_fee);
}

#[no_mangle]
pub extern "C" fn update_mint_params() {
    only_owner();
    let start_time: u64 = helpers::get_named_arg_with_user_errors(
        MINTING_START_TIME,
        Error::MissingMintingStart,
        Error::InvalidMintingStart,
    )
    .unwrap_or_revert();
    let start_time_r3: u64 = helpers::get_named_arg_with_user_errors(
        MINTING_START_TIME_R3,
        Error::MissingMintingStart,
        Error::InvalidMintingStart,
    )
    .unwrap_or_revert();
    let end_time: u64 = helpers::get_named_arg_with_user_errors(
        MINTING_END_TIME,
        Error::MissingMintingEnd,
        Error::InvalidMintingEnd,
    )
    .unwrap_or_revert();

    let minting_price: U256 = helpers::get_named_arg_with_user_errors(
        MINT_FEE,
        Error::MissingCsprMintFee,
        Error::InvalidCsprMintFee,
    )
    .unwrap_or_revert();

    let minting_price_r3: U256 = helpers::get_named_arg_with_user_errors(
        MINT_FEE_R3,
        Error::MissingCsprMintFee,
        Error::InvalidCsprMintFee,
    )
    .unwrap_or_revert();

    helpers::set_key(MINTING_START_TIME, start_time);
    helpers::set_key(MINTING_START_TIME_R3, start_time_r3);
    helpers::set_key(MINTING_END_TIME, end_time);
    helpers::set_key(MINT_FEE, minting_price);
    helpers::set_key(MINT_FEE_R3, minting_price_r3);
}

fn call_cep78_mint(nft_contract_package: &Key, owner: Key, count: u8) {
    let nft_contract_package_addr: HashAddr = nft_contract_package.into_hash().unwrap_or_revert();
    let nft_package_hash: ContractPackageHash = ContractPackageHash::new(nft_contract_package_addr);

    let metadata: String = (r#"{"name":"CasperPunks Mystery Box","symbol":"MBOX","token_uri":"https://api-gen0.casperpunks.io/lootbox.png","checksum":"8cb33616573675fe4f9c24638f397b068a815ac0dd79d5e7c86ccf845d66f233"}"#).to_string();

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
}

fn is_round2_done() -> bool {
    let start_time_r3: u64 = helpers::get_stored_value_with_user_errors(
        MINTING_START_TIME_R3,
        Error::MissingMintingStart,
        Error::InvalidMintingStart,
    );
    let current_time_sec = helpers::current_block_timestamp_sec();
    current_time_sec >= start_time_r3
}

pub fn minting_valid_time() {
    let start_time: u64 = helpers::get_stored_value_with_user_errors(
        MINTING_START_TIME,
        Error::MissingMintingStart,
        Error::InvalidMintingStart,
    );
    let end_time: u64 = helpers::get_stored_value_with_user_errors(
        MINTING_END_TIME,
        Error::MissingMintingEnd,
        Error::InvalidMintingEnd,
    );
    let current_time_sec = helpers::current_block_timestamp_sec();
    helpers::require(
        start_time <= current_time_sec && current_time_sec <= end_time,
        Error::MintingTimeInvalid,
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
