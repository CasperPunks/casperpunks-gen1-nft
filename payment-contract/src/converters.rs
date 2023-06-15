#![allow(unused_imports)]
#![allow(unused_parens)]
#![allow(non_snake_case)]

use std::convert::TryInto;
use types::{bytesrepr::ToBytes, ContractHash, Key, U256, U512};

pub fn u512_to_u256(nb: U512) -> U256 {
    let mut b = [0u8; 64];
    nb.to_big_endian(&mut b);
    U256::from_big_endian(&b[32..64])
}
