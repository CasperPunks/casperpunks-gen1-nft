use alloc::{boxed::Box, string::String, vec};

use crate::constants::*;

use casper_types::{
    CLType, CLTyped, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Parameter, U256,
};

fn transfer_owner() -> EntryPoint {
    EntryPoint::new(
        String::from(TRANSFER_OWNER_ENTRY_POINT_NAME),
        vec![Parameter::new(ARG_CONTRACT_OWNER, CLType::Key)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

fn change_fee_receiver() -> EntryPoint {
    EntryPoint::new(
        String::from(CHANGE_FEE_RECEIVER_ENTRY_POINT_NAME),
        vec![Parameter::new(ARG_FEE_RECEIVER, CLType::Key)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

fn change_mint_fee() -> EntryPoint {
    EntryPoint::new(
        String::from(CHANGE_MINT_FEE_ENTRY_POINT_NAME),
        vec![Parameter::new(ARG_MINT_FEE, CLType::U256)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

fn set_addresses_whitelist() -> EntryPoint {
    EntryPoint::new(
        String::from(SET_ADDRESSES_WHITELIST),
        vec![
            Parameter::new(
                ARG_NEW_ADDRESSES_WHITELIST,
                CLType::List(Box::new(CLType::Key)),
            ),
            Parameter::new(ARG_IS_WHITELIST, CLType::Bool),
        ],
        CLType::String,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}
fn update_addresses_whitelist() -> EntryPoint {
    EntryPoint::new(
        String::from(UPDATE_ADDRESSES_WHITELIST),
        vec![
            Parameter::new(
                ARG_NEW_ADDRESSES_WHITELIST,
                CLType::List(Box::new(CLType::Key)),
            ),
            Parameter::new(ARG_NUMBER_OF_TICKETS, CLType::U8),
        ],
        CLType::String,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

fn mint() -> EntryPoint {
    EntryPoint::new(
        String::from(MINT_ENTRY_POINT_NAME),
        vec![
            Parameter::new(ARG_NFT_CONTRACT_PACKAGE, CLType::Key),
            Parameter::new(ARG_TOKEN_META_DATA, CLType::String),
        ],
        CLType::String,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

fn init() -> EntryPoint {
    EntryPoint::new(
        String::from(INIT_ENTRY_POINT_NAME),
        vec![Parameter::new(ARG_CONTRACT_HASH, CLType::Key)],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

/// Returns the default set of ERC20 token entry points.
pub(crate) fn default() -> EntryPoints {
    let mut entry_points = EntryPoints::new();
    // entry_points.add_entry_point(transfer_dev());
    entry_points.add_entry_point(transfer_owner());
    entry_points.add_entry_point(change_fee_receiver());
    entry_points.add_entry_point(change_mint_fee());
    entry_points.add_entry_point(mint());
    entry_points.add_entry_point(set_addresses_whitelist());
    entry_points.add_entry_point(update_addresses_whitelist());
    entry_points.add_entry_point(init());
    entry_points
}
