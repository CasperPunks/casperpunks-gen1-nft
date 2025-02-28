use alloc::string::{String, ToString};
use casper_contract::contract_api::storage;
use casper_types::{contracts::NamedKeys, ContractPackageHash, Key, U256};

use crate::constants::*;
pub fn default(
    nft_factory_contract_name: String,
    contract_owner: Key,
    wcspr_mint_fee: U256,
    wcspr_mint_fee_r3: U256,
    contract_package_hash: ContractPackageHash,
    fee_receiver: Key,
    _fee_token: Option<Key>,
) -> NamedKeys {
    let mut named_keys = NamedKeys::new();

    // Contract 'Named keys'
    named_keys.insert(
        NFT_FACTORY_CONTRACT_KEY_NAME.to_string(),
        Key::from(storage::new_uref(nft_factory_contract_name).into_read()),
    );
    named_keys.insert(
        CONTRACT_OWNER_KEY_NAME.to_string(),
        Key::from(storage::new_uref(contract_owner)),
    );
    // named_keys.insert(DEV.to_string(), Key::from(storage::new_uref(dev)));
    named_keys.insert(
        FEE_RECEIVER.to_string(),
        Key::from(storage::new_uref(fee_receiver)),
    );

    named_keys.insert(
        MINT_FEE.to_string(),
        storage::new_uref(wcspr_mint_fee).into(),
    );

    named_keys.insert(
        MINT_FEE_R3.to_string(),
        storage::new_uref(wcspr_mint_fee_r3).into(),
    );

    named_keys.insert(
        "contract_package_hash".to_string(),
        storage::new_uref(contract_package_hash).into(),
    );

    named_keys
}
