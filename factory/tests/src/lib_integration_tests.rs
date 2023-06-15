use casper_engine_test_support::{
    ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_ACCOUNT_ADDR,
    DEFAULT_ACCOUNT_PUBLIC_KEY, DEFAULT_PAYMENT, DEFAULT_RUN_GENESIS_REQUEST,
    MINIMUM_ACCOUNT_CREATION_BALANCE,
};

use casper_types::{
    account::AccountHash, bytesrepr, bytesrepr::FromBytes, bytesrepr::ToBytes, crypto::SecretKey,
    runtime_args, system::mint, CLType, CLTyped, ContractHash, ContractPackageHash, Key, PublicKey,
    RuntimeArgs, StoredValue, URef, U128, U256, U512,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::convert::TryInto;

const EXAMPLE_ERC20_TOKEN: &str = "erc20_token.wasm";
const MARKET_PLACE_WASM: &str = "contract.wasm";
const CEP47_WASM: &str = "cep47-token.wasm";
const CEP78_WASM: &str = "cep78-contract.wasm";

// const CEP47_WASM: &str = "casper-punk-token.wasm";
const PAYMENT_WASM: &str = "payment_contract.wasm";
const TEST_SESSION_WASM: &str = "test-session.wasm";

const DEX_CONTRACT: &str = "contract.wasm";
const ERC20_TOKEN_CONTRACT_KEY: &str = "erc20_token_contract";
const ARG_NAME: &str = "name";
const ARG_SYMBOL: &str = "symbol";
const ARG_DECIMALS: &str = "decimals";
const ARG_TOTAL_SUPPLY: &str = "total_supply";
const ARG_NEW_MINTER: &str = "new_minter";
const RESULT_KEY: &str = "result";
const TOKEN_TOTAL_SUPPLY: u128 = 1_000_000_000_000_000_000_000_000_000;

pub const STABLESWAP_CONTRACT_NAME: &str = "stableswap_contract_name";
pub const ARG_POOLED_TOKENS: &str = "pooled_tokens";
pub const ARG_LP_TOKEN: &str = "lp_token";
pub const ARG_A: &str = "a";
pub const ARG_FEE: &str = "fee";
pub const ARG_ADMIN_FEE: &str = "admin_fee";
pub const ARG_CONTRACT_OWNER: &str = "contract_owner";

// #[derive(Serialize, Deserialize, Clone)]
// pub(crate) struct SellingInMarket {
//     token_id: String,
//     nft_contract: Key,
//     offeror: Option<Key>, //token seller
//     minimum_offer: U256,  // min price in WCSPR
//     is_active: bool,
//     bidder: Vec<Key>,
//     bidding_price: Vec<U256>,
// }

// impl ToBytes for SellingInMarket {
//     fn to_bytes(&self) -> Result<Vec<u8>, bytesrepr::Error> {
//         let mut result = bytesrepr::allocate_buffer(self)?;
//         result.extend(self.token_id.to_bytes()?);
//         result.extend(self.nft_contract.to_bytes()?);
//         result.extend(self.offeror.to_bytes()?);
//         result.extend(self.minimum_offer.to_bytes()?);
//         result.extend(self.is_active.to_bytes()?);
//         result.extend(self.bidder.to_bytes()?);
//         result.extend(self.bidding_price.to_bytes()?);
//         Ok(result)
//     }

//     fn serialized_length(&self) -> usize {
//         self.token_id.serialized_length()
//             + self.nft_contract.serialized_length()
//             + self.offeror.serialized_length()
//             + self.minimum_offer.serialized_length()
//             + self.is_active.serialized_length()
//             + self.bidder.serialized_length()
//             + self.bidding_price.serialized_length()
//     }
// }

// impl FromBytes for SellingInMarket {
//     fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
//         let (token_id, remainder) = String::from_bytes(bytes)?;
//         let (nft_contract, remainder) = Key::from_bytes(remainder)?;
//         let (offeror, remainder) = Option::<Key>::from_bytes(remainder)?;
//         let (minimum_offer, remainder) = U256::from_bytes(remainder)?;
//         let (is_active, remainder) = bool::from_bytes(remainder)?;
//         let (bidder, remainder) = Vec::<Key>::from_bytes(remainder)?;
//         let (bidding_price, remainder) = Vec::<U256>::from_bytes(remainder)?;

//         let ret = SellingInMarket {
//             token_id,
//             nft_contract,
//             offeror,       //token seller
//             minimum_offer, // min price in WCSPR
//             is_active,
//             bidder,
//             bidding_price,
//         };
//         Ok((ret, remainder))
//     }
// }

// impl CLTyped for SellingInMarket {
//     fn cl_type() -> CLType {
//         CLType::Any
//     }
// }

fn get_token_key_name(symbol: String) -> String {
    ERC20_TOKEN_CONTRACT_KEY.to_owned() + "_" + &symbol
}

fn get_account1_addr() -> AccountHash {
    let sk: SecretKey = SecretKey::secp256k1_from_bytes(&[221u8; 32]).unwrap();
    let pk: PublicKey = PublicKey::from(&sk);
    let a: AccountHash = pk.to_account_hash();
    a
}

fn get_account2_addr() -> AccountHash {
    let sk: SecretKey = SecretKey::secp256k1_from_bytes(&[212u8; 32]).unwrap();
    let pk: PublicKey = PublicKey::from(&sk);
    let a: AccountHash = pk.to_account_hash();
    a
}

fn get_bidders() -> (Vec<AccountHash>, Vec<PublicKey>) {
    let mut ret: Vec<AccountHash> = vec![];
    let mut pks: Vec<PublicKey> = vec![];
    for i in 0..50 {
        let sk: SecretKey = SecretKey::secp256k1_from_bytes(&[(i + 1) as u8; 32]).unwrap();
        let pk: PublicKey = PublicKey::from(&sk);
        let a: AccountHash = pk.to_account_hash();
        ret.push(a);
        pks.push(pk);
    }

    (ret, pks)
}

fn get_test_result<T: FromBytes + CLTyped>(
    builder: &mut InMemoryWasmTestBuilder,
    test_session: ContractPackageHash,
) -> T {
    let contract_package = builder
        .get_contract_package(test_session)
        .expect("should have contract package");
    let enabled_versions = contract_package.enabled_versions();
    let (_version, contract_hash) = enabled_versions
        .iter()
        .rev()
        .next()
        .expect("should have latest version");

    builder.get_value(*contract_hash, RESULT_KEY)
}

fn call_and_get<T: FromBytes + CLTyped>(
    builder: &mut InMemoryWasmTestBuilder,
    test_session: ContractPackageHash,
    func_name: &str,
    args: RuntimeArgs,
) -> T {
    let exec_request = ExecuteRequestBuilder::versioned_contract_call_by_hash(
        *DEFAULT_ACCOUNT_ADDR,
        test_session,
        None,
        func_name,
        args,
    )
    .build();
    builder.exec(exec_request).expect_success().commit();

    get_test_result(builder, test_session)
}
fn get_contract_purse_balance(
    builder: &mut InMemoryWasmTestBuilder,
    contract_hash: ContractHash,
) -> U512 {
    let contract_key = builder.get_contract(contract_hash).unwrap();

    // println!("contract_key {:?}", contract_key,);

    let mint_named_keys = contract_key.named_keys().clone();
    // println!("mint_named_keys {:?}", mint_named_keys,);

    let contract_purse: Key = mint_named_keys["contract_purse"];
    let purse = contract_purse.into_uref().unwrap();
    // println!("contract_purse {:?}", purse,);
    let balance = builder.get_purse_balance(purse);

    println!("balance {:?}", balance);
    balance
}

fn get_selling_in_market(
    builder: &mut InMemoryWasmTestBuilder,
    contract_hash: ContractHash,
    token_id: String,
) {
    let contract_key = builder.get_contract(contract_hash).unwrap();

    // println!("contract_key {:?}", contract_key,);

    let mint_named_keys = contract_key.named_keys().clone();
    // println!("mint_named_keys {:?}", mint_named_keys,);

    let selling_in_maket = mint_named_keys["selling_in_market"];
    let selling_in_maket_uref = selling_in_maket.into_uref().unwrap();

    let result = builder.query_dictionary_item(None, selling_in_maket_uref, &token_id);

    // let purse = contract_purse.into_uref().unwrap();
    // println!("contract_purse {:?}", purse,);
    // let balance = builder.get_purse_balance(purse);
    // let a = result.from_bytes(result);
    println!("balance {:?}", result);
}

/// Converts hash addr of Account into Hash, and Hash into Account
///
/// This is useful for making sure ERC20 library respects different variants of Key when storing
/// balances.
// fn invert_erc20_address(address: Key) -> Key {
//     match address {
//         Key::Account(account_hash) => Key::Hash(account_hash.value()),
//         Key::Hash(contract_hash) => Key::Account(AccountHash::new(contract_hash)),
//         _ => panic!("Unsupported Key variant"),
//     }
// }

#[derive(Copy, Clone)]
struct TestContext {
    nft_contract_hash: ContractHash,
    marketplace_hash: ContractHash,
    marketplace_package_hash: ContractPackageHash,
}

fn exec_call(
    builder: &mut InMemoryWasmTestBuilder,
    account_hash: AccountHash,
    contract_hash: ContractHash,
    fun_name: &str,
    args: RuntimeArgs,
    expect_success: bool,
) {
    let request =
        ExecuteRequestBuilder::contract_call_by_hash(account_hash, contract_hash, fun_name, args)
            .build();
    if expect_success {
        builder.exec(request).expect_success().commit();
    } else {
        builder.exec(request).expect_failure();
    }
}

fn bid_or_buy(
    builder: &mut InMemoryWasmTestBuilder,
    account_hash: AccountHash,
    args: RuntimeArgs,
    expect_success: bool,
) {
    let request = ExecuteRequestBuilder::standard(account_hash, PAYMENT_WASM, args).build();
    if expect_success {
        builder.exec(request).expect_success().commit();
    } else {
        builder.exec(request).expect_failure();
    }
}

fn setup() -> (InMemoryWasmTestBuilder, TestContext) {
    let mut builder = InMemoryWasmTestBuilder::default();
    builder.run_genesis(&*DEFAULT_RUN_GENESIS_REQUEST);

    let id: Option<u64> = None;
    let transfer_1_args = runtime_args! {
        mint::ARG_TARGET => get_account1_addr(),
        mint::ARG_AMOUNT => MINIMUM_ACCOUNT_CREATION_BALANCE,
        mint::ARG_ID => id,
    };
    let transfer_2_args = runtime_args! {
        mint::ARG_TARGET => get_account2_addr(),
        mint::ARG_AMOUNT => MINIMUM_ACCOUNT_CREATION_BALANCE,
        mint::ARG_ID => id,
    };

    let transfer_request_1 =
        ExecuteRequestBuilder::transfer(*DEFAULT_ACCOUNT_ADDR, transfer_1_args).build();
    let transfer_request_2 =
        ExecuteRequestBuilder::transfer(*DEFAULT_ACCOUNT_ADDR, transfer_2_args).build();

    let deploy_nft = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        CEP78_WASM,
        runtime_args! {
            "collection_name" => "Gen1 Punk".to_string(),
            "collection_symbol" => "Gen1".to_string(),
            "total_token_supply" => 10000u64,
            "contract_name" => "cep47".to_string(),
            "admin" => Key::from(*DEFAULT_ACCOUNT_ADDR)
        },
    )
    .build();

    builder.exec(transfer_request_1).expect_success().commit();
    builder.exec(transfer_request_2).expect_success().commit();
    builder.exec(deploy_nft).expect_success().commit();

    // transfer to bidders
    let (bidders, _) = get_bidders();
    for a in bidders {
        let transfer_args = runtime_args! {
            mint::ARG_TARGET => a,
            mint::ARG_AMOUNT => U512::from(1_000_000_000_000_000u128),
            mint::ARG_ID => id,
        };
        let transfer_request =
            ExecuteRequestBuilder::transfer(*DEFAULT_ACCOUNT_ADDR, transfer_args).build();
        builder.exec(transfer_request).expect_success().commit();
    }

    let account = builder
        .get_account(*DEFAULT_ACCOUNT_ADDR)
        .expect("should have account");
    let nft_contract_hash = account
        .named_keys()
        .get("cep47_contract_hash")
        .and_then(|key| key.into_hash())
        .map(ContractHash::new)
        .expect("should have contract hash");

    let deploy_market = ExecuteRequestBuilder::standard(
        *DEFAULT_ACCOUNT_ADDR,
        MARKET_PLACE_WASM,
        runtime_args! {
            "maket_name" => "marketcep47".to_string(),
            "contract_owner" => Key::from(*DEFAULT_ACCOUNT_ADDR),
            "market_fee_receiver" => Key::from(get_account1_addr()),
            "market_fee" => U256::from(50),
            "nft_contract_hash" => Key::from(nft_contract_hash)
        },
    )
    .build();
    builder.exec(deploy_market).expect_success().commit();

    let account = builder
        .get_account(*DEFAULT_ACCOUNT_ADDR)
        .expect("should have account");
    let marketplace_hash = account
        .named_keys()
        .get("marketcep47")
        .and_then(|key| key.into_hash())
        .map(ContractHash::new)
        .expect("should have contract hash");

    let marketplace_package_hash: ContractPackageHash =
        builder.get_value(marketplace_hash, "contract_package_hash");

    // exec_call(&mut builder, *DEFAULT_ACCOUNT_ADDR, usdc_token, "transfer", runtime_args! {
    //     "recipient" => Key::from(get_account1_addr()),
    //     "amount" => U256::from(1_000_000_000_000_000_000_000_000u128)
    // }, true);

    let null_hash: [u8; 32] = vec![0u8; 32].try_into().unwrap();
    // mint 100 nfts
    for i in 0..50 {
        exec_call(
            &mut builder,
            *DEFAULT_ACCOUNT_ADDR,
            nft_contract_hash,
            "mint",
            runtime_args! {
                "recipient" => Key::from(*DEFAULT_ACCOUNT_ADDR),
                "token_ids" => vec![i.to_string()],
                "token_metas" => vec![BTreeMap::<String, String>::new()]
            },
            true,
        );

        // approving
        exec_call(
            &mut builder,
            *DEFAULT_ACCOUNT_ADDR,
            nft_contract_hash,
            "approve",
            runtime_args! {
                "spender" => Key::from(marketplace_package_hash),
                "token_ids" => vec![i.to_string()]
            },
            true,
        );
    }

    // exec_call(&mut builder, *DEFAULT_ACCOUNT_ADDR, marketplace_hash, "set_support_token", runtime_args! {
    //     "nft_contract_hash" => Key::from(nft_contract_hash),
    //     "nft_enabled" => true
    // }, true);

    let tc = TestContext {
        nft_contract_hash,
        marketplace_hash,
        marketplace_package_hash,
    };

    (builder, tc)
}

// #[test]
// fn test_with_bid_then_increase_then_revoke() {
//     let (mut builder, tc) = setup();

//     //
//     // bidding
//     let (bidders, bidders_pubkey) = get_bidders();
//     let mut i = 0u128;
//     let arrr = [1, 5, 10, 15, 20, 25, 30, 35, 40, 45, 50];
//     for m in 0..arrr.len() {
//         for c in arrr[m]..arrr[m] + 1 {
//             println!("c =  {:?}", c);
//             exec_call(
//                 &mut builder,
//                 *DEFAULT_ACCOUNT_ADDR,
//                 tc.marketplace_hash,
//                 "sell",
//                 runtime_args! {
//                     "nft_contract_hash" => Key::from(tc.nft_contract_hash),
//                     "token_id" => (c - 1).to_string(),
//                     "minimum_offer" => U256::from(1500_000_000_000u128)
//                 },
//                 true,
//             );
//             println!(" sell token ID {:?}", c - 1);
//             println!(" sell cost {:?}", builder.last_exec_gas_cost());

//             for i in 1..c + 1 {
//                 println!(" i =  {:?}", i);
//                 let a = bidders[i];
//                 // println!("balance {:?}", builder.get_public_key_balance_result(bidders_pubkey[i as usize].clone()));
//                 assert_eq!(a, bidders_pubkey[i as usize].clone().to_account_hash());
//                 let bal = *builder
//                     .get_public_key_balance_result(bidders_pubkey[i as usize].clone())
//                     .motes()
//                     .unwrap();
//                 bid_or_buy(
//                     &mut builder,
//                     a,
//                     runtime_args! {
//                         "amount" => U512::from(120_000_000_000u128),
//                         "deposit_entry_point_name" => "bid",
//                         // "sell_index" => c as u64,
//                         "nft_contract_hash" => Key::from(tc.nft_contract_hash),
//                         "token_id" => (c - 1).to_string(),
//                         "bidder" => Key::from(a),
//                         "marketplace_hash" => Key::from(tc.marketplace_hash)
//                     },
//                     true,
//                 );
//                 println!(
//                     " BID with {:?} bid cost {:?}",
//                     i,
//                     builder.last_exec_gas_cost()
//                 );
//                 bid_or_buy(
//                     &mut builder,
//                     a,
//                     runtime_args! {
//                         "amount" => U512::from( (11_000_000_000) as u128),
//                         "deposit_entry_point_name" => "increase_bid",
//                         // "sell_index" => c as u64,
//                         "nft_contract_hash" => Key::from(tc.nft_contract_hash),
//                         "token_id" => (c - 1).to_string(),
//                         "bidder" => Key::from(a),
//                         "marketplace_hash" => Key::from(tc.marketplace_hash)
//                     },
//                     true,
//                 );
//                 println!(
//                     " increase_bid with {:?} bid cost {:?}",
//                     i,
//                     builder.last_exec_gas_cost()
//                 );

//                 // let bal_after = *builder
//                 //     .get_public_key_balance_result(bidders_pubkey[i as usize].clone())
//                 //     .motes()
//                 //     .unwrap();
//                 // assert_eq!(
//                 //     bal,
//                 //     bal_after + *DEFAULT_PAYMENT + U512::from(120_000_000_000u128)
//                 // );
//             }

//             // exec_call(
//             //     &mut builder,
//             //     *DEFAULT_ACCOUNT_ADDR,
//             //     tc.marketplace_hash,
//             //     "sell",
//             //     runtime_args! {
//             //         "nft_contract_hash" => Key::from(tc.nft_contract_hash),
//             //         "token_id" => (c - 1).to_string(),
//             //         "minimum_offer" => U256::from(110_000_000_000u128)
//             //     },
//             //     true,
//             // );
//             // println!(" sell token ID {:?}", c - 1);
//             // println!(" SELL with {:?} bid cost {:?}", c, builder.last_exec_gas_cost());

//             // exec_call(
//             //     &mut builder,
//             //     *DEFAULT_ACCOUNT_ADDR,
//             //     tc.marketplace_hash,
//             //     "accept_price",
//             //     runtime_args! {
//             //         // "nft_contract_hash" => Key::from(tc.nft_contract_hash),
//             //         "accepted_bidder" => Key::from(bidders[c]),
//             //         "accepted_price" => U256::from(110_000_000_000u128),
//             //         "token_id" => (c - 1).to_string()
//             //     },
//             //     true,
//             // );
//             // println!(
//             //     " ACCEPT PRICE with {:?} bid cost {:?}",
//             //     c,
//             //     builder.last_exec_gas_cost()
//             // );
//             // bid_or_buy(
//             //     &mut builder,
//             //     bidders[c - 2],
//             //     runtime_args! {
//             //         "amount" => U512::from(100_000_000_000u128 ),
//             //         "deposit_entry_point_name" => "bid",
//             //         // "sell_index" => c as u64,
//             //         "nft_contract_hash" => Key::from(tc.nft_contract_hash),
//             //         "token_id" => (c - 1).to_string(),
//             //         "bidder" => Key::from(bidders[c-2]),
//             //         "marketplace_hash" => Key::from(tc.marketplace_hash)
//             //     },
//             //     true,
//             // );
//             // println!(
//             //     " bid with {:?} bid cost {:?}",
//             //     i,
//             //     builder.last_exec_gas_cost()
//             // );
//             // bid_or_buy(
//             //     &mut builder,
//             //     bidders[c - 2],
//             //     runtime_args! {
//             //         "amount" => U512::from( (1500_000_000_000) as u128),
//             //         "deposit_entry_point_name" => "buy",
//             //         // "sell_index" => c as u64,
//             //         "nft_contract_hash" => Key::from(tc.nft_contract_hash),
//             //         "token_id" => (c - 1).to_string(),
//             //         "buyer" => Key::from(bidders[c-2]),
//             //         "marketplace_hash" => Key::from(tc.marketplace_hash)
//             //     },
//             //     true,
//             // );
//             // exec_call(
//             //     &mut builder,
//             //     *DEFAULT_ACCOUNT_ADDR,
//             //     tc.marketplace_hash,
//             //     "accept_price",
//             //     runtime_args! {
//             //         // "nft_contract_hash" => Key::from(tc.nft_contract_hash),
//             //         "accepted_bidder" => Key::from(bidders[c - 2]),
//             //         "accepted_price" => U256::from(110_000_000_000u128),
//             //         "token_id" => (c - 1).to_string()
//             //     },
//             //     true,
//             // );
//             // println!(
//             //     " change price with {:?} bid cost {:?}",
//             //     c,
//             //     builder.last_exec_gas_cost()
//             // );
//             // println!(
//             //     " buy with {:?} bid cost {:?}",
//             //     c - 1,
//             //     builder.last_exec_gas_cost()
//             // );
//             // exec_call(
//             //     &mut builder,
//             //     *DEFAULT_ACCOUNT_ADDR,
//             //     tc.marketplace_hash,
//             //     "sell",
//             //     runtime_args! {
//             //         "nft_contract_hash" => Key::from(tc.nft_contract_hash),
//             //         "token_id" => (c - 1).to_string(),
//             //         "minimum_offer" => U256::from(12_000_000_000u128)
//             //     },
//             //     true,
//             // );
//             // exec_call(
//             //     &mut builder,
//             //     *DEFAULT_ACCOUNT_ADDR,
//             //     tc.marketplace_hash,
//             //     "change_price",
//             //     runtime_args! {
//             //         // "nft_contract_hash" => Key::from(tc.nft_contract_hash),
//             //         "token_id" => (c - 1).to_string(),
//             //         "minimum_offer" => U256::from(110_000_000_000u128)
//             //     },
//             //     true,
//             // );
//             // println!(
//             //     " CHANGE PRICE token ID with {:?} bid cost {:?}",
//             //     c ,
//             //     builder.last_exec_gas_cost()
//             // );
//             // reading balance before bid
//             // let mut balances: Vec<U512> = vec![U512::from(0u128); c];
//             // for i in 0..c {
//             //     let a = bidders[i];
//             //     let b = builder.get_public_key_balance_result(bidders_pubkey[i as usize].clone());
//             //     balances[i] = *b.motes().unwrap();
//             // }
//             // println!("get_selling_in_market");
//             // get_selling_in_market(&mut builder, tc.marketplace_hash, "0".to_string());
//             // revoke
//             // exec_call(
//             //     &mut builder,
//             //     *DEFAULT_ACCOUNT_ADDR,
//             //     tc.marketplace_hash,
//             //     "accept_price",
//             //     runtime_args! {
//             //         // "nft_contract_hash" => Key::from(tc.nft_contract_hash),
//             //         "accepted_bidder" => Key::from(bidders[c - 2]),
//             //         "accepted_price" => U256::from(10_000_000_000u128 + ((c-2)*5_000_000_000 + 1_000_000_000) as u128),
//             //         "token_id" => (c - 1).to_string()
//             //     },
//             //     true,
//             // );
//             // println!(
//             //     " change price with {:?} bid cost {:?}",
//             //     c,
//             //     builder.last_exec_gas_cost()
//             // );
//             // let mut balances_after: Vec<U512> = vec![U512::from(0u128); c];
//             // for i in 0..c {
//             //     let a = bidders[i];
//             //     let b = builder.get_public_key_balance_result(bidders_pubkey[i as usize].clone());
//             //     balances_after[i] = *b.motes().unwrap();
//             //     assert_eq!(balances_after[i], balances[i]);
//             // }
//         }
//     }
//     // let vec_ids = ["1", "2", "3", "4"].to_vec();
//     // exec_call(
//     //     &mut builder,
//     //     *DEFAULT_ACCOUNT_ADDR,
//     //     tc.marketplace_hash,
//     //     "emergency_withdraw_nfts",
//     //     runtime_args! {
//     //         "nft_contract_hash" => Key::from(tc.nft_contract_hash),
//     //         "token_ids" => vec_ids,
//     //         // "minimum_offer" => U256::from(100_000_000_000u128)
//     //     },
//     //     true,
//     // );
//     // println!(" hi hi with cost {:?}", builder.last_exec_gas_cost());

//     // let balance = get_contract_purse_balance(&mut builder, tc.marketplace_hash);

//     // println!(" balance contract purse {:?}", balance);

//     // let bal_owner_before = *builder
//     //     .get_public_key_balance_result(DEFAULT_ACCOUNT_PUBLIC_KEY.clone())
//     //     .motes()
//     //     .unwrap();

//     // exec_call(
//     //     &mut builder,
//     //     *DEFAULT_ACCOUNT_ADDR,
//     //     tc.marketplace_hash,
//     //     "emergency_withdraw_cspr",
//     //     runtime_args! {
//     //         // "nft_contract_hash" => Key::from(tc.nft_contract_hash),
//     //         "amount" => balance,
//     //         // "minimum_offer" => U256::from(100_000_000_000u128)
//     //     },
//     //     true,
//     // );
//     // println!(" emergency cspr cost {:?}", builder.last_exec_gas_cost());
//     // let bal_owner_after = *builder
//     //     .get_public_key_balance_result(DEFAULT_ACCOUNT_PUBLIC_KEY.clone())
//     //     .motes()
//     //     .unwrap();
//     // assert_eq!(
//     //     bal_owner_before + balance - *DEFAULT_PAYMENT,
//     //     bal_owner_after
//     // );

//     // let contract_balances_after = get_contract_purse_balance(&mut builder, tc.marketplace_hash);
//     // assert_eq!(
//     //     contract_balances_after,
//     //     U512::zero()
//     // );
// }

#[test]
fn test_with_bid_then_increase_then_revoke() {
    let (mut builder, tc) = setup();

    //
    // bidding
    let (bidders, bidders_pubkey) = get_bidders();
    let mut i = 0u128;
    let arrr = [1, 5, 10, 15, 20, 25, 30, 35, 40, 45];
    for m in 0..arrr.len() {
        for c in arrr[m]..arrr[m] + 1 {
            println!("c =  {:?}", c);
            exec_call(
                &mut builder,
                *DEFAULT_ACCOUNT_ADDR,
                tc.marketplace_hash,
                "sell",
                runtime_args! {
                    "nft_contract_hash" => Key::from(tc.nft_contract_hash),
                    "token_id" => (c - 1).to_string(),
                    "minimum_offer" => U256::from(1500_000_000_000u128)
                },
                true,
            );
            println!(" sell token ID {:?}", c - 1);
            println!(" sell cost {:?}", builder.last_exec_gas_cost());

            for i in 1..c + 1 {
                println!(" i =  {:?}", i);
                let a = bidders[i];
                // println!("balance {:?}", builder.get_public_key_balance_result(bidders_pubkey[i as usize].clone()));
                assert_eq!(a, bidders_pubkey[i as usize].clone().to_account_hash());
                let bal = *builder
                    .get_public_key_balance_result(bidders_pubkey[i as usize].clone())
                    .motes()
                    .unwrap();
                bid_or_buy(
                    &mut builder,
                    a,
                    runtime_args! {
                        "amount" => U512::from(110_000_000_000u128 ),
                        "deposit_entry_point_name" => "bid",
                        // "sell_index" => c as u64,
                        "nft_contract_hash" => Key::from(tc.nft_contract_hash),
                        "token_id" => (c - 1).to_string(),
                        "bidder" => Key::from(a),
                        "marketplace_hash" => Key::from(tc.marketplace_hash)
                    },
                    true,
                );
                println!(
                    " bid with {:?} bid cost {:?}",
                    i,
                    builder.last_exec_gas_cost()
                );

                // let bal_after = *builder
                //     .get_public_key_balance_result(bidders_pubkey[i as usize].clone())
                //     .motes()
                //     .unwrap();
                // assert_eq!(
                //     bal,
                //     bal_after
                //         + *DEFAULT_PAYMENT
                //         + U512::from(10_000_000_000u128 + (i * 5_000_000_000) as u128)
                // );
                // bid_or_buy(
                //     &mut builder,
                //     a,
                //     runtime_args! {
                //         "amount" => U512::from( (1_000_000_000) as u128),
                //         "deposit_entry_point_name" => "increase_bid",
                //         // "sell_index" => c as u64,
                //         "nft_contract_hash" => Key::from(tc.nft_contract_hash),
                //         "token_id" => (c - 1).to_string(),
                //         "bidder" => Key::from(a),
                //         "marketplace_hash" => Key::from(tc.marketplace_hash)
                //     },
                //     true,
                // );
                // println!(
                //     " increase_bid with {:?} bid cost {:?}",
                //     i,
                //     builder.last_exec_gas_cost()
                // );
            }


            exec_call(
                &mut builder,
                bidders[c],
                tc.marketplace_hash,
                "revoke_bid",
                runtime_args! {
                    "nft_contract_hash" => Key::from(tc.nft_contract_hash),
                    "token_id" => (c - 1).to_string(),
                    // "minimum_offer" => U256::from(150_000_000_000u128)
                },
                true,
            );
            println!(" sell token ID {:?}", c - 1);
            println!(" REVBOKE BID cost {:?}", builder.last_exec_gas_cost());


            // exec_call(
            //     &mut builder,
            //     *DEFAULT_ACCOUNT_ADDR,
            //     tc.marketplace_hash,
            //     "revoke_sell",
            //     runtime_args! {
            //         "nft_contract_hash" => Key::from(tc.nft_contract_hash),
            //         "token_id" => (c - 1).to_string(),
            //         // "minimum_offer" => U256::from(150_000_000_000u128)
            //     },
            //     true,
            // );
            // println!(" sell token ID {:?}", c - 1);
            // println!(" REVBOKE SELLL cost {:?}", builder.last_exec_gas_cost());

            // bid_or_buy(
            //     &mut builder,
            //     bidders[c ],
            //     runtime_args! {
            //         "amount" => U512::from( (1500_000_000_000) as u128),
            //         "deposit_entry_point_name" => "buy",
            //         // "sell_index" => c as u64,
            //         "nft_contract_hash" => Key::from(tc.nft_contract_hash),
            //         "token_id" => (c - 1).to_string(),
            //         "buyer" => Key::from(bidders[c]),
            //         "marketplace_hash" => Key::from(tc.marketplace_hash)
            //     },
            //     true,
            // );

            // println!(
            //     " BUYBUY with {:?} bid cost {:?}",
            //     c ,
            //     builder.last_exec_gas_cost()
            // );

            // bid_or_buy(
            //     &mut builder,
            //     bidders[c],
            //     runtime_args! {
            //         "amount" => U512::from( (1000_000_000_000) as u128),
            //         "deposit_entry_point_name" => "increase_bid",
            //         // "sell_index" => c as u64,
            //         "nft_contract_hash" => Key::from(tc.nft_contract_hash),
            //         "token_id" => (c - 1).to_string(),
            //         "bidder" => Key::from(bidders[c]),
            //         "marketplace_hash" => Key::from(tc.marketplace_hash)
            //     },
            //     true,
            // );
            // println!(
            //     " Donesell increaseBID with {:?} bid cost {:?}",
            //     c,
            //     builder.last_exec_gas_cost()
            // );

            // bid_or_buy(
            //     &mut builder,
            //     bidders[c+1],
            //     runtime_args! {
            //         "amount" => U512::from(1000_000_000_000u128 ),
            //         "deposit_entry_point_name" => "bid",
            //         // "sell_index" => c as u64,
            //         "nft_contract_hash" => Key::from(tc.nft_contract_hash),
            //         "token_id" => (c - 1).to_string(),
            //         "bidder" => Key::from(bidders[c+1]),
            //         "marketplace_hash" => Key::from(tc.marketplace_hash)
            //     },
            //     true,
            // );
            // println!(
            //     " bid DoneSELL with {:?} bid cost {:?}",
            //     c,
            //     builder.last_exec_gas_cost()
            // );

            // bid_or_buy(
            //     &mut builder,
            //     bidders[c - 2],
            //     runtime_args! {
            //         "amount" => U512::from(100_000_000_000u128 ),
            //         "deposit_entry_point_name" => "bid",
            //         // "sell_index" => c as u64,
            //         "nft_contract_hash" => Key::from(tc.nft_contract_hash),
            //         "token_id" => (c - 1).to_string(),
            //         "bidder" => Key::from(bidders[c-2]),
            //         "marketplace_hash" => Key::from(tc.marketplace_hash)
            //     },
            //     true,
            // );
            // println!(
            //     " bid with {:?} bid cost {:?}",
            //     i,
            //     builder.last_exec_gas_cost()
            // );

            // bid_or_buy(
            //     &mut builder,
            //     bidders[c - 2],
            //     runtime_args! {
            //         "amount" => U512::from( (1500_000_000_000) as u128),
            //         "deposit_entry_point_name" => "buy",
            //         // "sell_index" => c as u64,
            //         "nft_contract_hash" => Key::from(tc.nft_contract_hash),
            //         "token_id" => (c - 1).to_string(),
            //         "buyer" => Key::from(bidders[c-2]),
            //         "marketplace_hash" => Key::from(tc.marketplace_hash)
            //     },
            //     true,
            // );

            println!(
                " buy with {:?} bid cost {:?}",
                c - 1,
                builder.last_exec_gas_cost()
            );

            // exec_call(
            //     &mut builder,
            //     *DEFAULT_ACCOUNT_ADDR,
            //     tc.marketplace_hash,
            //     "sell",
            //     runtime_args! {
            //         "nft_contract_hash" => Key::from(tc.nft_contract_hash),
            //         "token_id" => (c - 1).to_string(),
            //         "minimum_offer" => U256::from(12_000_000_000u128)
            //     },
            //     true,
            // );

            // exec_call(
            //     &mut builder,
            //     *DEFAULT_ACCOUNT_ADDR,
            //     tc.marketplace_hash,
            //     "change_price",
            //     runtime_args! {
            //         // "nft_contract_hash" => Key::from(tc.nft_contract_hash),
            //         "token_id" => (c - 1).to_string(),
            //         "minimum_offer" => U256::from(12_000_000_000u128)
            //     },
            //     true,
            // );

            // println!(
            //     " change price token ID with {:?} bid cost {:?}",
            //     c - 1,
            //     builder.last_exec_gas_cost()
            // );

            // reading balance before bid
            // let mut balances: Vec<U512> = vec![U512::from(0u128); c];
            // for i in 0..c {
            //     let a = bidders[i];
            //     let b = builder.get_public_key_balance_result(bidders_pubkey[i as usize].clone());
            //     balances[i] = *b.motes().unwrap();
            // }

            // println!("get_selling_in_market");
            // get_selling_in_market(&mut builder, tc.marketplace_hash, "0".to_string());

            // revoke

            // exec_call(
            //     &mut builder,
            //     *DEFAULT_ACCOUNT_ADDR,
            //     tc.marketplace_hash,
            //     "accept_price",
            //     runtime_args! {
            //         // "nft_contract_hash" => Key::from(tc.nft_contract_hash),
            //         "accepted_bidder" => Key::from(bidders[c - 2]),
            //         "accepted_price" => U256::from(10_000_000_000u128 + ((c-2)*5_000_000_000 + 1_000_000_000) as u128),
            //         "token_id" => (c - 1).to_string()
            //     },
            //     true,
            // );
            // println!(
            //     " change price with {:?} bid cost {:?}",
            //     c,
            //     builder.last_exec_gas_cost()
            // );

            // let mut balances_after: Vec<U512> = vec![U512::from(0u128); c];
            // for i in 0..c {
            //     let a = bidders[i];
            //     let b = builder.get_public_key_balance_result(bidders_pubkey[i as usize].clone());
            //     balances_after[i] = *b.motes().unwrap();
            //     assert_eq!(balances_after[i], balances[i]);
            // }
        }
    }

    // let vec_ids = ["1", "2", "3", "4"].to_vec();
    // exec_call(
    //     &mut builder,
    //     *DEFAULT_ACCOUNT_ADDR,
    //     tc.marketplace_hash,
    //     "emergency_withdraw_nfts",
    //     runtime_args! {
    //         "nft_contract_hash" => Key::from(tc.nft_contract_hash),
    //         "token_ids" => vec_ids,
    //         // "minimum_offer" => U256::from(100_000_000_000u128)
    //     },
    //     true,
    // );
    // println!(" hi hi with cost {:?}", builder.last_exec_gas_cost());

    // let balance = get_contract_purse_balance(&mut builder, tc.marketplace_hash);

    // println!(" balance contract purse {:?}", balance);

    // let bal_owner_before = *builder
    //     .get_public_key_balance_result(DEFAULT_ACCOUNT_PUBLIC_KEY.clone())
    //     .motes()
    //     .unwrap();

    // exec_call(
    //     &mut builder,
    //     *DEFAULT_ACCOUNT_ADDR,
    //     tc.marketplace_hash,
    //     "emergency_withdraw_cspr",
    //     runtime_args! {
    //         // "nft_contract_hash" => Key::from(tc.nft_contract_hash),
    //         "amount" => balance,
    //         // "minimum_offer" => U256::from(100_000_000_000u128)
    //     },
    //     true,
    // );
    // println!(" emergency cspr cost {:?}", builder.last_exec_gas_cost());
    // let bal_owner_after = *builder
    //     .get_public_key_balance_result(DEFAULT_ACCOUNT_PUBLIC_KEY.clone())
    //     .motes()
    //     .unwrap();
    // assert_eq!(
    //     bal_owner_before + balance - *DEFAULT_PAYMENT,
    //     bal_owner_after
    // );

    // let contract_balances_after = get_contract_purse_balance(&mut builder, tc.marketplace_hash);
    // assert_eq!(
    //     contract_balances_after,
    //     U512::zero()
    // );
}

// // test
// #[test]
// fn test_emergency_withdraw() {
//     let (mut builder, tc) = setup();

//     //
//     // bidding
//     let (bidders, bidders_pubkey) = get_bidders();
//     let mut i = 0u128;
//     for c in 1..6 {
//         exec_call(
//             &mut builder,
//             *DEFAULT_ACCOUNT_ADDR,
//             tc.marketplace_hash,
//             "sell",
//             runtime_args! {
//                 "nft_contract_hash" => Key::from(tc.nft_contract_hash),
//                 "token_id" => (c - 1).to_string(),
//                 "minimum_offer" => U256::from(150_000_000_000u128)
//             },
//             true,
//         );
//         println!(
//             " sell token ID {:?}",
//             c -1
//         );

//         for i in 1..c {
//             let a = bidders[i];
//             // println!("balance {:?}", builder.get_public_key_balance_result(bidders_pubkey[i as usize].clone()));
//             assert_eq!(a, bidders_pubkey[i as usize].clone().to_account_hash());
//             let bal = *builder
//                 .get_public_key_balance_result(bidders_pubkey[i as usize].clone())
//                 .motes()
//                 .unwrap();
//             bid_or_buy(
//                 &mut builder,
//                 a,
//                 runtime_args! {
//                     "amount" => U512::from(10_000_000_000u128 + ( i * 5_000_000_000 ) as u128),
//                     "deposit_entry_point_name" => "bid",
//                     // "sell_index" => c as u64,
//                     "nft_contract_hash" => Key::from(tc.nft_contract_hash),
//                     "token_id" => (c - 1).to_string(),
//                     "bidder" => Key::from(a),
//                     "marketplace_hash" => Key::from(tc.marketplace_hash)
//                 },
//                 true,
//             );
//             // println!(
//             //     " bid with {:?} bid cost {:?}",
//             //     c,
//             //     10_000_000_000u128 + (i * 5_000_000_000) as u128
//             // );

//             let bal_after = *builder
//                 .get_public_key_balance_result(bidders_pubkey[i as usize].clone())
//                 .motes()
//                 .unwrap();
//             assert_eq!(
//                 bal,
//                 bal_after
//                     + *DEFAULT_PAYMENT
//                     + U512::from(10_000_000_000u128 + (i * 5_000_000_000) as u128)
//             );

//             // bid_or_buy(
//             //     &mut builder,
//             //     a,
//             //     runtime_args! {
//             //         "amount" => U512::from( (1_000_000_000) as u128),
//             //         "deposit_entry_point_name" => "increase_bid",
//             //         // "sell_index" => c as u64,
//             //         "nft_contract_hash" => Key::from(tc.nft_contract_hash),
//             //         "token_id" => (c - 1).to_string(),
//             //         "bidder" => Key::from(a),
//             //         "marketplace_hash" => Key::from(tc.marketplace_hash)
//             //     },
//             //     true,
//             // );
//             // println!(
//             //     " sell with {:?} bid cost {:?}",
//             //     c,
//             //     builder.last_exec_gas_cost()
//             // );
//         }

//         // reading balance before bid
//         let mut balances: Vec<U512> = vec![U512::from(0u128); c];
//         for i in 0..c {
//             let a = bidders[i];
//             let b = builder.get_public_key_balance_result(bidders_pubkey[i as usize].clone());
//             balances[i] = *b.motes().unwrap();
//         }

//         // println!("get_selling_in_market");
//         // get_selling_in_market(&mut builder, tc.marketplace_hash, "0".to_string());

//         // revoke

//         // exec_call(
//         //     &mut builder,
//         //     *DEFAULT_ACCOUNT_ADDR,
//         //     tc.marketplace_hash,
//         //     "accept_price",
//         //     runtime_args! {
//         //         // "nft_contract_hash" => Key::from(tc.nft_contract_hash),
//         //         "accepted_bidder" => Key::from(bidders[c - 2]),
//         //         "accepted_price" => U256::from(10_000_000_000u128 + ((c-2)*1000000000) as u128),
//         //         "token_id" => (c - 1).to_string()
//         //     },
//         //     true,
//         // );
//         // println!(
//         //     " change price with {:?} bid cost {:?}",
//         //     c,
//         //     builder.last_exec_gas_cost()
//         // );

//         // let mut balances_after: Vec<U512> = vec![U512::from(0u128); c];
//         // for i in 0..c {
//         //     let a = bidders[i];
//         //     let b = builder.get_public_key_balance_result(bidders_pubkey[i as usize].clone());
//         //     balances_after[i] = *b.motes().unwrap();
//         //     assert_eq!(balances_after[i], balances[i]);
//         // }
//     }
//     let vec_ids = ["1", "2", "3", "4"].to_vec();
//     exec_call(
//         &mut builder,
//         *DEFAULT_ACCOUNT_ADDR,
//         tc.marketplace_hash,
//         "emergency_withdraw_nfts",
//         runtime_args! {
//             "nft_contract_hash" => Key::from(tc.nft_contract_hash),
//             "token_ids" => vec_ids,
//             // "minimum_offer" => U256::from(100_000_000_000u128)
//         },
//         true,
//     );
//     println!(" hi hi with cost {:?}", builder.last_exec_gas_cost());

//     let balance = get_contract_purse_balance(&mut builder, tc.marketplace_hash);

//     println!(" balance contract purse {:?}", balance);

//     let bal_owner_before = *builder
//         .get_public_key_balance_result(DEFAULT_ACCOUNT_PUBLIC_KEY.clone())
//         .motes()
//         .unwrap();

//     exec_call(
//         &mut builder,
//         *DEFAULT_ACCOUNT_ADDR,
//         tc.marketplace_hash,
//         "emergency_withdraw_cspr",
//         runtime_args! {
//             // "nft_contract_hash" => Key::from(tc.nft_contract_hash),
//             "amount" => balance,
//             // "minimum_offer" => U256::from(100_000_000_000u128)
//         },
//         true,
//     );
//     println!(" emergency cspr cost {:?}", builder.last_exec_gas_cost());
//     let bal_owner_after = *builder
//         .get_public_key_balance_result(DEFAULT_ACCOUNT_PUBLIC_KEY.clone())
//         .motes()
//         .unwrap();
//     assert_eq!(
//         bal_owner_before + balance - *DEFAULT_PAYMENT,
//         bal_owner_after
//     );

//     let contract_balances_after = get_contract_purse_balance(&mut builder, tc.marketplace_hash);
//     assert_eq!(
//         contract_balances_after,
//         U512::zero()
//     );
// }

// #[test]
// fn test_with_bid_then_revoke() {
//     let (mut builder, tc) = setup();

//     // bidding
//     let (bidders, bidders_pubkey) = get_bidders();
//     let mut i = 0u128;
//     for c in 3..bidders.len() {
//         for i in 0..c {
//             let a = bidders[i];
//             // println!("balance {:?}", builder.get_public_key_balance_result(bidders_pubkey[i as usize].clone()));
//             assert_eq!(a, bidders_pubkey[i as usize].clone().to_account_hash());
//             let bal = *builder
//                 .get_public_key_balance_result(bidders_pubkey[i as usize].clone())
//                 .motes()
//                 .unwrap();
//             bid_or_buy(
//                 &mut builder,
//                 a,
//                 runtime_args! {
//                     "amount" => U512::from(10_000_000_000u128 + (i*1000000000) as u128),
//                     "deposit_entry_point_name" => "bid",
//                     // "sell_index" => c as u64,
//                     "nft_contract_hash" => Key::from(tc.nft_contract_hash),
//                     "token_id" => (c - 1).to_string(),
//                     "bidder" => Key::from(a),
//                     "marketplace_hash" => Key::from(tc.marketplace_hash)
//                 },
//                 true,
//             );
//             let bal_after = *builder
//                 .get_public_key_balance_result(bidders_pubkey[i as usize].clone())
//                 .motes()
//                 .unwrap();
//             // let gas = builder.last_exec_gas_cost().value();
//             assert_eq!(
//                 bal,
//                 bal_after
//                     + *DEFAULT_PAYMENT
//                     + U512::from(10_000_000_000u128 + (i * 1000000000) as u128)
//             );
//         }

//         // reading balance before bid
//         let mut balances: Vec<U512> = vec![U512::from(0u128); c];
//         for i in 0..c {
//             let a = bidders[i];
//             let b = builder.get_public_key_balance_result(bidders_pubkey[i as usize].clone());
//             balances[i] = *b.motes().unwrap();
//         }

//         exec_call(
//             &mut builder,
//             *DEFAULT_ACCOUNT_ADDR,
//             tc.marketplace_hash,
//             "sell",
//             runtime_args! {
//                 "nft_contract_hash" => Key::from(tc.nft_contract_hash),
//                 "token_id" => (c - 1).to_string(),
//                 "minimum_offer" => U256::from(100_000_000_000u128)
//             },
//             true,
//         );
//         println!(
//             " sell with {:?} bid cost {:?}",
//             c,
//             builder.last_exec_gas_cost()
//         );

//         // revoke

//         exec_call(
//             &mut builder,
//             *DEFAULT_ACCOUNT_ADDR,
//             tc.marketplace_hash,
//             "accept_price",
//             runtime_args! {
//                 // "nft_contract_hash" => Key::from(tc.nft_contract_hash),
//                 "accepted_bidder" => Key::from(bidders[c - 2]),
//                 "accepted_price" => U256::from(10_000_000_000u128 + ((c-2)*1000000000) as u128),
//                 "token_id" => (c - 1).to_string()
//             },
//             true,
//         );
//         println!(
//             " change price with {:?} bid cost {:?}",
//             c,
//             builder.last_exec_gas_cost()
//         );

//         // let mut balances_after: Vec<U512> = vec![U512::from(0u128); c];
//         // for i in 0..c {
//         //     let a = bidders[i];
//         //     let b = builder.get_public_key_balance_result(bidders_pubkey[i as usize].clone());
//         //     balances_after[i] = *b.motes().unwrap();
//         //     assert_eq!(balances_after[i], balances[i]);
//         // }
//     }
// }

// #[test]
// fn test_with_bid_then_increase_then_revoke() {
//     let (mut builder, tc) = setup();

//     //
//     // bidding
//     let (bidders, bidders_pubkey) = get_bidders();
//     let mut i = 0u128;
//     for c in 2..6 {
//         exec_call(
//             &mut builder,
//             *DEFAULT_ACCOUNT_ADDR,
//             tc.marketplace_hash,
//             "sell",
//             runtime_args! {
//                 "nft_contract_hash" => Key::from(tc.nft_contract_hash),
//                 "token_id" => (c - 1).to_string(),
//                 "minimum_offer" => U256::from(150_000_000_000u128)
//             },
//             true,
//         );
//         println!(
//             " sell token ID {:?}",
//             c -1
//         );

//         for i in 1..c {
//             let a = bidders[i];
//             // println!("balance {:?}", builder.get_public_key_balance_result(bidders_pubkey[i as usize].clone()));
//             assert_eq!(a, bidders_pubkey[i as usize].clone().to_account_hash());
//             let bal = *builder
//                 .get_public_key_balance_result(bidders_pubkey[i as usize].clone())
//                 .motes()
//                 .unwrap();
//             bid_or_buy(
//                 &mut builder,
//                 a,
//                 runtime_args! {
//                     "amount" => U512::from(10_000_000_000u128 + ( i * 5_000_000_000 ) as u128),
//                     "deposit_entry_point_name" => "bid",
//                     // "sell_index" => c as u64,
//                     "nft_contract_hash" => Key::from(tc.nft_contract_hash),
//                     "token_id" => (c - 1).to_string(),
//                     "bidder" => Key::from(a),
//                     "marketplace_hash" => Key::from(tc.marketplace_hash)
//                 },
//                 true,
//             );
//             // println!(
//             //     " bid with {:?} bid cost {:?}",
//             //     c,
//             //     10_000_000_000u128 + (i * 5_000_000_000) as u128
//             // );

//             let bal_after = *builder
//                 .get_public_key_balance_result(bidders_pubkey[i as usize].clone())
//                 .motes()
//                 .unwrap();
//             assert_eq!(
//                 bal,
//                 bal_after
//                     + *DEFAULT_PAYMENT
//                     + U512::from(10_000_000_000u128 + (i * 5_000_000_000) as u128)
//             );

//             bid_or_buy(
//                 &mut builder,
//                 a,
//                 runtime_args! {
//                     "amount" => U512::from( (1_000_000_000) as u128),
//                     "deposit_entry_point_name" => "increase_bid",
//                     // "sell_index" => c as u64,
//                     "nft_contract_hash" => Key::from(tc.nft_contract_hash),
//                     "token_id" => (c - 1).to_string(),
//                     "bidder" => Key::from(a),
//                     "marketplace_hash" => Key::from(tc.marketplace_hash)
//                 },
//                 true,
//             );
//             println!(
//                 " sell with {:?} bid cost {:?}",
//                 c,
//                 builder.last_exec_gas_cost()
//             );
//         }

//         // reading balance before bid
//         let mut balances: Vec<U512> = vec![U512::from(0u128); c];
//         for i in 0..c {
//             let a = bidders[i];
//             let b = builder.get_public_key_balance_result(bidders_pubkey[i as usize].clone());
//             balances[i] = *b.motes().unwrap();
//         }

//         // println!("get_selling_in_market");
//         // get_selling_in_market(&mut builder, tc.marketplace_hash, "0".to_string());

//         // revoke

//         // exec_call(
//         //     &mut builder,
//         //     *DEFAULT_ACCOUNT_ADDR,
//         //     tc.marketplace_hash,
//         //     "accept_price",
//         //     runtime_args! {
//         //         // "nft_contract_hash" => Key::from(tc.nft_contract_hash),
//         //         "accepted_bidder" => Key::from(bidders[c - 2]),
//         //         "accepted_price" => U256::from(10_000_000_000u128 + ((c-2)*5_000_000_000 + 1_000_000_000) as u128),
//         //         "token_id" => (c - 1).to_string()
//         //     },
//         //     true,
//         // );
//         // println!(
//         //     " change price with {:?} bid cost {:?}",
//         //     c,
//         //     builder.last_exec_gas_cost()
//         // );

//         // let mut balances_after: Vec<U512> = vec![U512::from(0u128); c];
//         // for i in 0..c {
//         //     let a = bidders[i];
//         //     let b = builder.get_public_key_balance_result(bidders_pubkey[i as usize].clone());
//         //     balances_after[i] = *b.motes().unwrap();
//         //     assert_eq!(balances_after[i], balances[i]);
//         // }
//     }
//     // let vec_ids = ["1", "2", "3", "4"].to_vec();
//     // exec_call(
//     //     &mut builder,
//     //     *DEFAULT_ACCOUNT_ADDR,
//     //     tc.marketplace_hash,
//     //     "emergency_withdraw_nfts",
//     //     runtime_args! {
//     //         "nft_contract_hash" => Key::from(tc.nft_contract_hash),
//     //         "token_ids" => vec_ids,
//     //         // "minimum_offer" => U256::from(100_000_000_000u128)
//     //     },
//     //     true,
//     // );
//     // println!(" hi hi with cost {:?}", builder.last_exec_gas_cost());

//     // let balance = get_contract_purse_balance(&mut builder, tc.marketplace_hash);

//     // println!(" balance contract purse {:?}", balance);

//     // let bal_owner_before = *builder
//     //     .get_public_key_balance_result(DEFAULT_ACCOUNT_PUBLIC_KEY.clone())
//     //     .motes()
//     //     .unwrap();

//     // exec_call(
//     //     &mut builder,
//     //     *DEFAULT_ACCOUNT_ADDR,
//     //     tc.marketplace_hash,
//     //     "emergency_withdraw_cspr",
//     //     runtime_args! {
//     //         // "nft_contract_hash" => Key::from(tc.nft_contract_hash),
//     //         "amount" => balance,
//     //         // "minimum_offer" => U256::from(100_000_000_000u128)
//     //     },
//     //     true,
//     // );
//     // println!(" emergency cspr cost {:?}", builder.last_exec_gas_cost());
//     // let bal_owner_after = *builder
//     //     .get_public_key_balance_result(DEFAULT_ACCOUNT_PUBLIC_KEY.clone())
//     //     .motes()
//     //     .unwrap();
//     // assert_eq!(
//     //     bal_owner_before + balance - *DEFAULT_PAYMENT,
//     //     bal_owner_after
//     // );

//     // let contract_balances_after = get_contract_purse_balance(&mut builder, tc.marketplace_hash);
//     // assert_eq!(
//     //     contract_balances_after,
//     //     U512::zero()
//     // );
// }

// #[test]
// fn test_emergency_withdraw() {
//     let (mut builder, tc) = setup();

//     //
//     // bidding
//     let (bidders, bidders_pubkey) = get_bidders();
//     let mut i = 0u128;
//     for c in 1..6 {
//         exec_call(
//             &mut builder,
//             *DEFAULT_ACCOUNT_ADDR,
//             tc.marketplace_hash,
//             "sell",
//             runtime_args! {
//                 "nft_contract_hash" => Key::from(tc.nft_contract_hash),
//                 "token_id" => (c - 1).to_string(),
//                 "minimum_offer" => U256::from(150_000_000_000u128)
//             },
//             true,
//         );
//         println!(
//             " sell token ID {:?}",
//             c -1
//         );

//         for i in 1..c {
//             let a = bidders[i];
//             // println!("balance {:?}", builder.get_public_key_balance_result(bidders_pubkey[i as usize].clone()));
//             assert_eq!(a, bidders_pubkey[i as usize].clone().to_account_hash());
//             let bal = *builder
//                 .get_public_key_balance_result(bidders_pubkey[i as usize].clone())
//                 .motes()
//                 .unwrap();
//             bid_or_buy(
//                 &mut builder,
//                 a,
//                 runtime_args! {
//                     "amount" => U512::from(10_000_000_000u128 + ( i * 5_000_000_000 ) as u128),
//                     "deposit_entry_point_name" => "bid",
//                     // "sell_index" => c as u64,
//                     "nft_contract_hash" => Key::from(tc.nft_contract_hash),
//                     "token_id" => (c - 1).to_string(),
//                     "bidder" => Key::from(a),
//                     "marketplace_hash" => Key::from(tc.marketplace_hash)
//                 },
//                 true,
//             );
//             // println!(
//             //     " bid with {:?} bid cost {:?}",
//             //     c,
//             //     10_000_000_000u128 + (i * 5_000_000_000) as u128
//             // );

//             let bal_after = *builder
//                 .get_public_key_balance_result(bidders_pubkey[i as usize].clone())
//                 .motes()
//                 .unwrap();
//             assert_eq!(
//                 bal,
//                 bal_after
//                     + *DEFAULT_PAYMENT
//                     + U512::from(10_000_000_000u128 + (i * 5_000_000_000) as u128)
//             );

//             // bid_or_buy(
//             //     &mut builder,
//             //     a,
//             //     runtime_args! {
//             //         "amount" => U512::from( (1_000_000_000) as u128),
//             //         "deposit_entry_point_name" => "increase_bid",
//             //         // "sell_index" => c as u64,
//             //         "nft_contract_hash" => Key::from(tc.nft_contract_hash),
//             //         "token_id" => (c - 1).to_string(),
//             //         "bidder" => Key::from(a),
//             //         "marketplace_hash" => Key::from(tc.marketplace_hash)
//             //     },
//             //     true,
//             // );
//             // println!(
//             //     " sell with {:?} bid cost {:?}",
//             //     c,
//             //     builder.last_exec_gas_cost()
//             // );
//         }

//         // reading balance before bid
//         let mut balances: Vec<U512> = vec![U512::from(0u128); c];
//         for i in 0..c {
//             let a = bidders[i];
//             let b = builder.get_public_key_balance_result(bidders_pubkey[i as usize].clone());
//             balances[i] = *b.motes().unwrap();
//         }

//         // println!("get_selling_in_market");
//         // get_selling_in_market(&mut builder, tc.marketplace_hash, "0".to_string());

//         // revoke

//         // exec_call(
//         //     &mut builder,
//         //     *DEFAULT_ACCOUNT_ADDR,
//         //     tc.marketplace_hash,
//         //     "accept_price",
//         //     runtime_args! {
//         //         // "nft_contract_hash" => Key::from(tc.nft_contract_hash),
//         //         "accepted_bidder" => Key::from(bidders[c - 2]),
//         //         "accepted_price" => U256::from(10_000_000_000u128 + ((c-2)*1000000000) as u128),
//         //         "token_id" => (c - 1).to_string()
//         //     },
//         //     true,
//         // );
//         // println!(
//         //     " change price with {:?} bid cost {:?}",
//         //     c,
//         //     builder.last_exec_gas_cost()
//         // );

//         // let mut balances_after: Vec<U512> = vec![U512::from(0u128); c];
//         // for i in 0..c {
//         //     let a = bidders[i];
//         //     let b = builder.get_public_key_balance_result(bidders_pubkey[i as usize].clone());
//         //     balances_after[i] = *b.motes().unwrap();
//         //     assert_eq!(balances_after[i], balances[i]);
//         // }
//     }
//     let vec_ids = ["1", "2", "3", "4"].to_vec();
//     exec_call(
//         &mut builder,
//         *DEFAULT_ACCOUNT_ADDR,
//         tc.marketplace_hash,
//         "emergency_withdraw_nfts",
//         runtime_args! {
//             "nft_contract_hash" => Key::from(tc.nft_contract_hash),
//             "token_ids" => vec_ids,
//             // "minimum_offer" => U256::from(100_000_000_000u128)
//         },
//         true,
//     );
//     println!(" hi hi with cost {:?}", builder.last_exec_gas_cost());

//     let balance = get_contract_purse_balance(&mut builder, tc.marketplace_hash);

//     println!(" balance contract purse {:?}", balance);

//     let bal_owner_before = *builder
//         .get_public_key_balance_result(DEFAULT_ACCOUNT_PUBLIC_KEY.clone())
//         .motes()
//         .unwrap();

//     exec_call(
//         &mut builder,
//         *DEFAULT_ACCOUNT_ADDR,
//         tc.marketplace_hash,
//         "emergency_withdraw_cspr",
//         runtime_args! {
//             // "nft_contract_hash" => Key::from(tc.nft_contract_hash),
//             "amount" => balance,
//             // "minimum_offer" => U256::from(100_000_000_000u128)
//         },
//         true,
//     );
//     println!(" emergency cspr cost {:?}", builder.last_exec_gas_cost());
//     let bal_owner_after = *builder
//         .get_public_key_balance_result(DEFAULT_ACCOUNT_PUBLIC_KEY.clone())
//         .motes()
//         .unwrap();
//     assert_eq!(
//         bal_owner_before + balance - *DEFAULT_PAYMENT,
//         bal_owner_after
//     );

//     let contract_balances_after = get_contract_purse_balance(&mut builder, tc.marketplace_hash);
//     assert_eq!(
//         contract_balances_after,
//         U512::zero()
//     );
// }
