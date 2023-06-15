use alloc::{boxed::Box, string::String, vec};

use crate::constants::*;

use casper_types::{
    CLType, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Parameter,
};

fn redeem() -> EntryPoint {
    EntryPoint::new(
        String::from("redeem"),
        vec![Parameter::new("token_ids", CLType::List(Box::new(CLType::Key)))],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

fn init() -> EntryPoint {
    EntryPoint::new(
        String::from(INIT_ENTRY_POINT_NAME),
        vec![
            Parameter::new("contract_hash", CLType::Key),
            Parameter::new("box_package_hash", CLType::Key),
            Parameter::new("punk_gen1_package_hash", CLType::Key),
            Parameter::new("contract_owner", CLType::Key)
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

/// Returns the default set of ERC20 token entry points.
pub(crate) fn default() -> EntryPoints {
    let mut entry_points = EntryPoints::new();
    // entry_points.add_entry_point(transfer_dev());
    entry_points.add_entry_point(redeem());
    entry_points.add_entry_point(init());
    entry_points
}
