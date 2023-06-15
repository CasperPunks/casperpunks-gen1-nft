#![allow(unused_parens)]
#![allow(non_snake_case)]

extern crate alloc;
mod error;

use contract::{
    contract_api::{runtime}
};
use types::{
    runtime_args, ContractPackageHash, Key,
    RuntimeArgs
};

#[no_mangle]
pub extern "C" fn call() {
    let box_package_hash: Key = runtime::get_named_arg("box_package_hash");
    let redeem_package_hash: Key = runtime::get_named_arg("redeem_package_hash");
    let redeem_contract_hash: Key = runtime::get_named_arg("redeem_contract_hash");
    let token_ids: Vec<u64> = runtime::get_named_arg("token_ids");

    // first, approve
    let _: () = runtime::call_versioned_contract(
        ContractPackageHash::new(box_package_hash.into_hash().unwrap()),
        None,
        "set_approval_for_all",
        runtime_args! {
            "approve_all" => true,
            "operator" => redeem_contract_hash
        }
    );

    // redeem
    let _: () = runtime::call_versioned_contract(
        ContractPackageHash::new(redeem_package_hash.into_hash().unwrap()),
        None,
        "redeem",
        runtime_args! {
            "token_ids" => token_ids
        }
    );
}
