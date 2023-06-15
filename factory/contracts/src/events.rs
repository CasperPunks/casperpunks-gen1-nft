#![allow(unused_parens)]
#![allow(non_snake_case)]
#![allow(dead_code)]

// use std::collections::BTreeMap;
extern crate alloc;
use alloc::{
    collections::BTreeMap,
    string::{String, ToString},
    vec::*,
};
use casper_contract::contract_api::storage;
use casper_types::{account::AccountHash, ContractPackageHash, Key, URef, U256};

use crate::helpers::get_key;

pub enum FactoryEvent {
    Approval {
        owner: Key,
        spender: Key,
        value: U256,
    },
    Transfer {
        from: Key,
        to: Key,
        value: U256,
    },
    MintFactory {
        owner: Key,
        minted: u8,
    },
    Withdrawal {
        cspr_recipient: AccountHash,
        from: Key,
        value: U256,
    },
}

impl FactoryEvent {
    pub fn type_name(&self) -> String {
        match self {
            FactoryEvent::Approval {
                owner: _,
                spender: _,
                value: _,
            } => "approve",
            FactoryEvent::Transfer {
                from: _,
                to: _,
                value: _,
            } => "transfer",
            FactoryEvent::MintFactory {
                owner: _,
                minted: _,
            } => "mint_factory",
            FactoryEvent::Withdrawal {
                cspr_recipient: _,
                from: _,
                value: _,
            } => "withdrawal",
        }
        .to_string()
    }
}

pub fn contract_package_hash() -> ContractPackageHash {
    get_key::<ContractPackageHash>("contract_package_hash").unwrap()
}

pub(crate) fn emit(pair_event: &FactoryEvent) {
    let mut events = Vec::new();
    let package = contract_package_hash();
    match pair_event {
        FactoryEvent::Approval {
            owner,
            spender,
            value,
        } => {
            let mut event = BTreeMap::new();
            event.insert("contract_package_hash", package.to_string());
            event.insert("event_type", pair_event.type_name());
            event.insert("owner", owner.to_string());
            event.insert("spender", spender.to_string());
            event.insert("value", value.to_string());
            events.push(event);
        }
        FactoryEvent::Transfer { from, to, value } => {
            let mut event = BTreeMap::new();
            event.insert("contract_package_hash", package.to_string());
            event.insert("event_type", pair_event.type_name());
            event.insert("from", from.to_string());
            event.insert("to", to.to_string());
            event.insert("value", value.to_string());
            events.push(event);
        }
        FactoryEvent::MintFactory {
            owner,
            minted,
            // token_id,
        } => {
            let mut event = BTreeMap::new();
            event.insert("contract_package_hash", package.to_string());
            event.insert("event_type", pair_event.type_name());
            event.insert("owner", owner.to_string());
            event.insert("minted", minted.to_string());
            // event.insert("value", token.to_string());
            events.push(event);
        }
        FactoryEvent::Withdrawal {
            cspr_recipient,
            from,
            value,
        } => {
            let mut event = BTreeMap::new();
            event.insert("contract_package_hash", package.to_string());
            event.insert("event_type", pair_event.type_name());
            event.insert("cspr_recipient", cspr_recipient.to_string());
            event.insert("from", from.to_string());
            event.insert("value", value.to_string());
            events.push(event);
        }
    };
    for event in events {
        let _: URef = storage::new_uref(event);
    }
}
