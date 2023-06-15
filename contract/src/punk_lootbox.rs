use crate::{error::NFTCoreError, utils};
use alloc::{string::String, vec, vec::*};
use casper_contract::{
    contract_api::{runtime, storage },
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{CLType, EntryPoint, EntryPointAccess, EntryPointType, Key };

pub const THE_CONTRACT_OWNER: &str = "the_contract_owner";

pub fn only_owner() {
    utils::require(
        owner_internal() == utils::get_verified_caller().unwrap_or_revert(),
        NFTCoreError::OnlyOwner,
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

pub fn init(contract_owner: Key) {
    runtime::put_key(THE_CONTRACT_OWNER, storage::new_uref(contract_owner).into());
}

pub fn entry_points() -> Vec<EntryPoint> {
    vec![
        EntryPoint::new(
            String::from("transfer_owner"),
            vec![],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        )
    ]
}
