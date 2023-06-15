#![allow(unused_parens)]
#![allow(non_snake_case)]

extern crate alloc;
mod converters;
mod error;

use crate::{converters::u512_to_u256, error::Error};
use contract::{
    contract_api::{account, runtime, system},
    unwrap_or_revert::UnwrapOrRevert,
};
use types::{runtime_args, ContractHash, HashAddr, Key, RuntimeArgs, URef, U256, U512};

#[no_mangle]
pub extern "C" fn call() {
    let deposit_amount: U512 = runtime::get_named_arg("amount");

    let deposit_entry_point_name: String = runtime::get_named_arg("deposit_entry_point_name");

    let new_purse = system::create_purse();
    system::transfer_from_purse_to_purse(
        account::get_main_purse(),
        new_purse,
        deposit_amount,
        None,
    )
    .unwrap_or_revert_with(Error::ExcessiveAmount);

    let deposit_entry_point_args = retrieve_deposit_entry_point_name_args(
        deposit_entry_point_name.clone(),
        new_purse,
        u512_to_u256(deposit_amount),
    );

    if deposit_entry_point_args.is_empty() {
        runtime::revert(Error::InvalidDepositEntryPointName);
    }

    let box_contract_hash: Key = runtime::get_named_arg("box_contract_hash");

    let contract_hash_addr: HashAddr = box_contract_hash.into_hash().unwrap_or_revert();
    let box_contract_hash: ContractHash = ContractHash::new(contract_hash_addr);

    let count: u64 = runtime::get_named_arg("count");
    for _u in 0..count {
        let _: (String, Key, String) = runtime::call_contract(
            box_contract_hash,
            &deposit_entry_point_name,
            deposit_entry_point_args.clone(),
        );
    }
}

fn retrieve_deposit_entry_point_name_args(
    deposit_entry_point_name: String,
    purse: URef,
    _amount: U256,
) -> RuntimeArgs {
    if deposit_entry_point_name == "mint" {
        retrieve_mint_args(purse)
    } else {
        runtime_args! {}
    }
}

fn retrieve_mint_args(src_purse: URef) -> RuntimeArgs {
    runtime_args! {
        "token_meta_data" => runtime::get_named_arg::<String>("token_metadata"),
        "src_purse" => src_purse,
        "token_owner" => runtime::get_named_arg::<Key>("token_owner"),
    }
}
