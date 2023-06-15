#![allow(unused_parens)]
#![allow(non_snake_case)]

extern crate alloc;
mod error;

use contract::{
    contract_api::{runtime}
};
use types::{
    runtime_args, ContractPackageHash, Key,
    RuntimeArgs, URef
};

#[no_mangle]
pub extern "C" fn call() {
    let nft_package_hash: Key = runtime::get_named_arg("nft_package_hash");
    let token_ids: Vec<u64> = runtime::get_named_arg("token_ids");
    let target_key: Key = runtime::get_named_arg("target_key");
    let source_key: Key = runtime::get_named_arg("source_key");

    let _: (String, URef) = runtime::call_versioned_contract(
        ContractPackageHash::new(nft_package_hash.into_hash().unwrap()),
        None,
        "register_owner",
        runtime_args! {
            "token_owner" => target_key
        }
    );

    for token_id in &token_ids {
        let _: (String, Key) = runtime::call_versioned_contract(
            ContractPackageHash::new(nft_package_hash.into_hash().unwrap()),
            None,
            "transfer",
            runtime_args! {
                "source_key" => source_key,
                "target_key" => target_key,
                "token_id" => token_id.clone()
            }
        );
    }    
}
