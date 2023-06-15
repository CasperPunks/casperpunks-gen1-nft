use crate::{error::NFTCoreError, helpers, utils};
use alloc::{string::String, vec, vec::*};
use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{CLType, EntryPoint, EntryPointAccess, EntryPointType, Key};

pub const THE_CONTRACT_OWNER: &str = "the_contract_owner";
pub const THE_CONTRACT_MINTER: &str = "the_contract_minter";

pub fn only_owner() {
    utils::require(
        owner_internal() == utils::get_verified_caller().unwrap_or_revert(),
        NFTCoreError::OnlyOwner,
    );
}

pub fn only_owner_or_minter() {
    let owner: Key = owner_internal();
    let minter: Key = minter_internal();
    let caller: Key = helpers::get_immediate_caller_key();
    if caller != minter && caller != owner {
        runtime::revert(NFTCoreError::OnlyOwner);
    }
}

pub fn owner_internal() -> Key {
    let owner_key: Key = utils::get_stored_value_with_user_errors::<Key>(
        THE_CONTRACT_OWNER,
        NFTCoreError::MissingContractOwner,
        NFTCoreError::InvalidContractOwner,
    );
    owner_key
}

pub fn only_minter() {
    utils::require(
        minter_internal() == helpers::get_immediate_caller_key(),
        NFTCoreError::OnlyOwner,
    );
}

pub fn minter_internal() -> Key {
    let minter_key: Key = utils::get_stored_value_with_user_errors::<Key>(
        THE_CONTRACT_MINTER,
        NFTCoreError::MissingContractOwner,
        NFTCoreError::InvalidContractOwner,
    );
    minter_key
}

#[no_mangle]
pub extern "C" fn transfer_owner() {
    only_owner();
    let contract_owner: Key = runtime::get_named_arg(THE_CONTRACT_OWNER);
    utils::set_key(THE_CONTRACT_OWNER, contract_owner);
}

#[no_mangle]
pub extern "C" fn change_minter() {
    only_owner();
    let new_minter: Key = utils::get_named_arg_with_user_errors(
        THE_CONTRACT_MINTER,
        NFTCoreError::MissingContractOwner,
        NFTCoreError::InvalidContractOwner,
    )
    .unwrap_or_revert();
    utils::set_key(THE_CONTRACT_MINTER, new_minter);
}

pub fn init(contract_owner: Key) {
    runtime::put_key(THE_CONTRACT_OWNER, storage::new_uref(contract_owner).into());
    let contract_minter: Key = utils::get_named_arg_with_user_errors(
        THE_CONTRACT_MINTER,
        NFTCoreError::MissingContractOwner,
        NFTCoreError::InvalidContractOwner,
    )
    .unwrap_or_revert();
    runtime::put_key(
        THE_CONTRACT_MINTER,
        storage::new_uref(contract_minter).into(),
    );
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
            String::from("change_minter"),
            vec![],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        ),
    ]
}
