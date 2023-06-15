use crate::{address::Address, error::NFTCoreError};
use casper_contract::{contract_api::runtime, unwrap_or_revert::UnwrapOrRevert};
use casper_types::{self, system::CallStackElement, Key};

// Helper functions

pub(crate) fn get_key_from_address(addr: &Address) -> Key {
    match *addr {
        Address::Account(acc) => Key::from(acc),
        Address::Contract(contract_package_hash) => Key::from(contract_package_hash),
    }
}

/// Gets the immediate call stack element of the current execution.
fn get_immediate_call_stack_item() -> Option<CallStackElement> {
    let call_stack = runtime::get_call_stack();
    call_stack.into_iter().rev().nth(1)
}

/// Returns address based on a [`CallStackElement`].
///
/// For `Session` and `StoredSession` variants it will return account hash, and for `StoredContract`
/// case it will use contract hash as the address.
fn call_stack_element_to_address(call_stack_element: CallStackElement) -> Address {
    match call_stack_element {
        CallStackElement::Session { account_hash } => Address::from(account_hash),
        CallStackElement::StoredSession { account_hash, .. } => {
            // Stored session code acts in account's context, so if stored session wants to interact
            // with an ERC20 token caller's address will be used.
            Address::from(account_hash)
        }
        CallStackElement::StoredContract {
            contract_package_hash,
            ..
        } => Address::from(contract_package_hash),
    }
}

/// Gets the immediate session caller of the current execution.
///
/// This function ensures that only session code can execute this function, and disallows stored
/// session/stored contracts.
pub(crate) fn get_immediate_caller_address() -> Result<Address, NFTCoreError> {
    get_immediate_call_stack_item()
        .map(call_stack_element_to_address)
        .ok_or(NFTCoreError::InvalidContext)
}

pub(crate) fn get_immediate_caller_key() -> Key {
    let addr = get_immediate_caller_address().unwrap_or_revert();
    get_key_from_address(&addr)
}
