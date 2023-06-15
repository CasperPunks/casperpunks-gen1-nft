use crate::{address::Address, alloc::string::*, error::NFTCoreError};
use alloc::{string::String, vec::Vec};
use casper_contract::{
    contract_api::{self, runtime, storage},
    ext_ffi,
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    api_error,
    bytesrepr::{
        FromBytes, ToBytes, {self},
    },
    system::CallStackElement,
    ApiError, CLTyped, ContractPackageHash, Key, URef, U256, U512,
};
use core::{
    convert::{TryFrom, TryInto},
    mem::MaybeUninit,
    u64,
};

use crate::constants::*;

use crate::error;

// Helper functions

pub(crate) fn get_key<T: FromBytes + CLTyped>(name: &str) -> Option<T> {
    match runtime::get_key(name) {
        None => None,
        Some(value) => {
            let key = value.try_into().unwrap_or_revert();
            let result = storage::read(key).unwrap_or_revert().unwrap_or_revert();
            Some(result)
        }
    }
}

pub(crate) fn get_key_from_address(addr: &Address) -> Key {
    let self_key = match *addr {
        Address::Account(acc) => Key::from(acc),
        Address::Contract(contract_package_hash) => Key::from(contract_package_hash),
    };
    self_key
}

pub(crate) fn get_self_key() -> Key {
    let self_addr = get_self_address().unwrap_or_revert();
    return get_key_from_address(&self_addr);
}

pub(crate) fn get_self_address() -> Result<Address, NFTCoreError> {
    get_last_call_stack_item()
        .map(call_stack_element_to_address)
        .ok_or(NFTCoreError::InvalidContext)
}

fn get_last_call_stack_item() -> Option<CallStackElement> {
    let call_stack = runtime::get_call_stack();
    call_stack.into_iter().rev().nth(0)
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

pub(crate) fn to_ptr<T: ToBytes>(t: T) -> (*const u8, usize, Vec<u8>) {
    let bytes = t.into_bytes().unwrap_or_revert();
    let ptr = bytes.as_ptr();
    let size = bytes.len();
    (ptr, size, bytes)
}
pub(crate) fn read_host_buffer(size: usize) -> Result<Vec<u8>, ApiError> {
    let mut dest: Vec<u8> = if size == 0 {
        Vec::new()
    } else {
        let bytes_non_null_ptr = contract_api::alloc_bytes(size);
        unsafe { Vec::from_raw_parts(bytes_non_null_ptr.as_ptr(), size, size) }
    };
    read_host_buffer_into(&mut dest)?;
    Ok(dest)
}
pub(crate) fn read_host_buffer_into(dest: &mut [u8]) -> Result<usize, ApiError> {
    let mut bytes_written = MaybeUninit::uninit();
    let ret = unsafe {
        ext_ffi::casper_read_host_buffer(dest.as_mut_ptr(), dest.len(), bytes_written.as_mut_ptr())
    };
    // NOTE: When rewriting below expression as `result_from(ret).map(|_| unsafe { ... })`, and the
    // caller ignores the return value, execution of the contract becomes unstable and ultimately
    // leads to `Unreachable` error.
    api_error::result_from(ret)?;
    Ok(unsafe { bytes_written.assume_init() })
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
