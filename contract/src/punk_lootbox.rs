use crate::{constants::WHITELISTED_USERS, error::NFTCoreError, utils, utils::u256_to_u512};
use alloc::{string::String, vec, vec::*};
use casper_contract::{
    contract_api::{runtime, storage, system::transfer_from_purse_to_account},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{CLType, EntryPoint, EntryPointAccess, EntryPointType, Key, URef, U256};

pub const MINTING_START_TIME: &str = "minting_start_time";
pub const MINTING_END_TIME: &str = "minting_end_time";
pub const MINTING_PRICE: &str = "minting_price";
pub const WHITE_LIST_MAP: &str = "whitelist_map";
pub const CSPR_RECEIVER: &str = "cspr_receiver";
pub const THE_CONTRACT_OWNER: &str = "the_contract_owner";
pub const ARG_SRC_PURSE: &str = "src_purse";

pub fn only_owner() {
    utils::require(
        owner_internal() == utils::get_verified_caller().unwrap_or_revert(),
        NFTCoreError::OnlyOwner,
    );
}

pub fn only_whitelisted(recipient: Key) {
    let dict_key = utils::encode_dictionary_item_key(recipient);
    let v = utils::get_dictionary_value_from_key::<bool>(WHITE_LIST_MAP, &dict_key);
    utils::require(
        v.is_some() && v.unwrap() == true,
        NFTCoreError::NotWhitelisted,
    );
}

pub fn owner_internal() -> Key {
    let owner_key: Key = utils::get_stored_value_with_user_errors::<Key>(
        THE_CONTRACT_OWNER,
        NFTCoreError::MissingContractOwner,
        NFTCoreError::InvalidContractOwner,
    );
    owner_key
}

#[no_mangle]
pub extern "C" fn transfer_owner() {
    only_owner();
    let contract_owner: Key = runtime::get_named_arg(THE_CONTRACT_OWNER);
    utils::set_key(THE_CONTRACT_OWNER, contract_owner);
}

#[no_mangle]
pub extern "C" fn update_mint_params() {
    only_owner();
    let start_time: u64 = utils::get_named_arg_with_user_errors(
        MINTING_START_TIME,
        NFTCoreError::MissingMintingStart,
        NFTCoreError::InvalidMintingStart,
    )
    .unwrap_or_revert();
    let end_time: u64 = utils::get_named_arg_with_user_errors(
        MINTING_END_TIME,
        NFTCoreError::MissingMintingEnd,
        NFTCoreError::InvalidMintingEnd,
    )
    .unwrap_or_revert();

    let minting_price: U256 = utils::get_named_arg_with_user_errors(
        MINTING_PRICE,
        NFTCoreError::MissingMintingPrice,
        NFTCoreError::InvalidMintingPrice,
    )
    .unwrap_or_revert();

    utils::set_key(MINTING_START_TIME, start_time);
    utils::set_key(MINTING_END_TIME, end_time);
    utils::set_key(MINTING_PRICE, minting_price);
}

#[no_mangle]
pub extern "C" fn set_whitelist() {
    only_owner();
    let users: Vec<Key> = runtime::get_named_arg(WHITELISTED_USERS);
    for user in &users {
        let dict_key = utils::encode_dictionary_item_key(user.clone());
        utils::upsert_dictionary_value_from_key::<bool>(WHITE_LIST_MAP, &dict_key, true);
    }
}

pub fn minting_valid_time() {
    let start_time: u64 = utils::get_stored_value_with_user_errors(
        MINTING_START_TIME,
        NFTCoreError::MissingMintingStart,
        NFTCoreError::InvalidMintingStart,
    );
    let end_time: u64 = utils::get_stored_value_with_user_errors(
        MINTING_END_TIME,
        NFTCoreError::MissingMintingEnd,
        NFTCoreError::InvalidMintingEnd,
    );
    let current_time_sec = utils::current_block_timestamp_sec();
    utils::require(
        start_time <= current_time_sec && current_time_sec <= end_time,
        NFTCoreError::MintingTimeInvalid,
    );
}

pub fn take_cspr_from_minting() {
    let src_purse: URef = utils::get_named_arg_with_user_errors::<URef>(
        ARG_SRC_PURSE,
        NFTCoreError::MissingSrcPurse,
        NFTCoreError::InvalidSrcPurse,
    )
    .unwrap_or_revert();

    let price: U256 = utils::get_stored_value_with_user_errors(
        MINTING_PRICE,
        NFTCoreError::MissingMintingPrice,
        NFTCoreError::InvalidMintingPrice,
    );
    let cspr_receiver: Key = utils::get_stored_value_with_user_errors(
        CSPR_RECEIVER,
        NFTCoreError::MissingCSPRReceiver,
        NFTCoreError::InvalidCSPRReceiver,
    );
    let fee_receiver_pubkey = cspr_receiver.into_account().unwrap();
    transfer_from_purse_to_account(src_purse, fee_receiver_pubkey, u256_to_u512(price), None)
        .unwrap_or_revert_with(NFTCoreError::CanNotTransferCSPR);
}

pub fn minting_price_satisfied(provided_value: U256) {
    let minting_price: U256 = utils::get_stored_value_with_user_errors(
        MINTING_PRICE,
        NFTCoreError::MissingMintingPrice,
        NFTCoreError::InvalidMintingPrice,
    );
    utils::require(
        minting_price <= provided_value,
        NFTCoreError::MintingUnderPay,
    );
}

pub fn init(contract_owner: Key) {
    runtime::put_key(THE_CONTRACT_OWNER, storage::new_uref(contract_owner).into());

    let end_time: u64 = utils::get_named_arg_with_user_errors(
        MINTING_END_TIME,
        NFTCoreError::MissingMintingEnd,
        NFTCoreError::InvalidMintingEnd,
    )
    .unwrap_or_revert();
    let start_time: u64 = utils::get_named_arg_with_user_errors(
        MINTING_START_TIME,
        NFTCoreError::MissingMintingStart,
        NFTCoreError::InvalidMintingStart,
    )
    .unwrap_or_revert();

    let minting_price: U256 = utils::get_named_arg_with_user_errors(
        MINTING_PRICE,
        NFTCoreError::MissingMintingPrice,
        NFTCoreError::InvalidMintingPrice,
    )
    .unwrap_or_revert();

    let cspr_receiver: Key = utils::get_named_arg_with_user_errors(
        CSPR_RECEIVER,
        NFTCoreError::MissingCSPRReceiver,
        NFTCoreError::InvalidCSPRReceiver,
    )
    .unwrap_or_revert();

    runtime::put_key(MINTING_START_TIME, storage::new_uref(start_time).into());
    runtime::put_key(MINTING_END_TIME, storage::new_uref(end_time).into());
    runtime::put_key(MINTING_PRICE, storage::new_uref(minting_price).into());
    runtime::put_key(CSPR_RECEIVER, storage::new_uref(cspr_receiver).into());

    storage::new_dictionary(WHITE_LIST_MAP)
        .unwrap_or_revert_with(NFTCoreError::FailedToCreateDictionary);
}

pub fn entry_points() -> Vec<EntryPoint> {
    vec![
        EntryPoint::new(
            String::from("transfer_owner"),
            vec![],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        ),
        EntryPoint::new(
            String::from("update_mint_params"),
            vec![],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        ),
        EntryPoint::new(
            String::from("set_whitelist"),
            vec![],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        ),
    ]
}
