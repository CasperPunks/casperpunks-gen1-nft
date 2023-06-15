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
use types::{
    account::AccountHash, runtime_args, ContractHash, ContractPackageHash, HashAddr, Key,
    RuntimeArgs, URef, U256, U512,
};

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

    let factory_contract_hash: Key = runtime::get_named_arg("factory_contract_hash");

    let contract_hash_addr: HashAddr = factory_contract_hash.into_hash().unwrap_or_revert();
    let factory: ContractHash = ContractHash::new(contract_hash_addr);

    let _: () =
        runtime::call_contract(factory, &deposit_entry_point_name, deposit_entry_point_args);
}

fn retrieve_deposit_entry_point_name_args(
    deposit_entry_point_name: String,
    purse: URef,
    amount: U256,
) -> RuntimeArgs {
    if deposit_entry_point_name == "mint" {
        retrieve_mint_args(purse, amount)
    } else {
        runtime_args! {}
    }
}

fn retrieve_mint_args(src_purse: URef, amount: U256) -> RuntimeArgs {
    runtime_args! {
        "token_meta_data" => runtime::get_named_arg::<String>("token_metadata"),
        "src_purse" => src_purse,
        "amount" => amount,
        "token_owner" => runtime::get_named_arg::<Key>("token_owner"),
        "nft_contract_package" => runtime::get_named_arg::<Key>("nft_contract_package"),
        "count" => runtime::get_named_arg::<u8>("count"),
    }
}
