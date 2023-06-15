require('dotenv').config()
const fs = require('fs');
const {
  DeployUtil,
  Keys,
  CasperClient,
  RuntimeArgs,
  CLString,
  CLU64,
  CLU8,
  CLU256,
  CLOption,
  CLBool,
  CLAccountHash,
  CLByteArray,
  CLKey,
  CLValueBuilder,
} = require("casper-js-sdk");
const Utils = require("./indexC.js");
const {
  utils,
  helpers,
  CasperContractClient,
} = require("casper-js-client-helper");
const { installContract, contractSimpleGetter, createRecipientAddress } = helpers;
const { sleep, getDeploy } = require('./utils')
let key = require('./keys.json').key
const {
  NODE_ADDRESS,
  EVENT_STREAM_ADDRESS,
  CHAIN_NAME,
  WASM_PATH
} = process.env
let paymentAmount = '140000000000' //140
let privateKeyPem = `
-----BEGIN PRIVATE KEY-----
${key}
-----END PRIVATE KEY-----
`

let privateKeyBuffer = Keys.Ed25519.parsePrivateKey(Keys.Ed25519.readBase64WithPEM(privateKeyPem))
let publicKey = Keys.Ed25519.privateToPublicKey(Uint8Array.from(privateKeyBuffer))
let KEYS = new Keys.Ed25519.parseKeyPair(publicKey, Uint8Array.from(privateKeyBuffer))
console.log('pubkey', KEYS.accountHex())


const main = async () => {

  //   --session-arg "collection_name:string=''" \
  const collection_name = new CLString("Mystery Box");
  // --session-arg "collection_symbol:string=''" \
  const collection_symbol = new CLString("MBOX");
  // --session-arg "total_token_supply:u64='10'" \
  const total_token_supply = new CLU64("10000");
  // --------------------
  // |    Mode      | u8 |
  // --------------------
  // | Minter       | 0  |
  // --------------------
  // | Assigned     | 1  |
  // --------------------
  // | Transferable | 2  |
  // --------------------
  //
  // --session-arg "ownership_mode:u8='2'" \
  //
  const ownership_mode = new CLU8("2");
  //
  // --------------------
  // |   NFTKind   | u8 |
  // --------------------
  // | Physical    | 0  |
  // --------------------
  // | Digital     | 1  |
  // --------------------
  // | Virtual     | 2  |
  // --------------------
  //
  // --session-arg "nft_kind:u8='1'" \
  //
  const nft_kind = new CLU8(1);
  //
  // --------------------
  // | NFTHolderMode | u8 |
  // --------------------
  // | Accounts      | 0  |
  // --------------------
  // | Contracts     | 1  |
  // --------------------
  // | Mixed         | 2  |
  // --------------------
  //
  // --session-arg "holder_mode:opt_u8='2'" \
  const holder_mode = new CLU8(2);
  //const holder_mode = new CLU8(2);


  // --session-arg "holder_mode:opt_u8='2'" \
  const events_mode = new CLU8(2) //new CLOption(Some(new CLU8(1)));
  //const holder_mode = new CLU8(2);


  // owner_reverse_lookup_mode

  const owner_reverse_lookup_mode = new CLU8(1) //new CLOption(Some(new CLU8(1)));


  //
  // --------------------
  // | MintingMode | u8 |
  // --------------------
  // | Installer   | 0  |
  // --------------------
  // | Public      | 1  |
  // --------------------
  //
  const minting_mode = new CLU8(1);

  // Optional
  //
  // --session-arg "json_schema:string='nft-schema'" \
  const json_schema = new CLString("nft-schema");
  //
  // allows minting when true
  //
  // --session-arg "allow_minting:bool='true'" \
  //
  const allow_minting = new CLBool(true);
  //
  // --------------------------
  // | NFTMetadataKind  | u8 |
  // -------------------------
  // | CEP78            | 0  |
  // -------------------------
  // | NFT721           | 1  |
  // -------------------------
  // | Raw              | 2  |
  // -------------------------
  // | CustomValidated  | 3  |
  // --------------------------
  // | CasperPunk       | 4  |
  // --------------------------
  //
  // == CEP-78 metadata example
  // {
  // "name": "John Doe",
  // "token_uri": "https://www.barfoo.com",
  // "checksum": "940bffb3f2bba35f84313aa26da09ece3ad47045c6a1292c2bbd2df4ab1a55fb"
  // }
  // ==
  // --session-arg "nft_metadata_kind:u8='1'" \
  //
  const nft_metadata_kind = new CLU8(0);
  //
  // --------------------------
  // | NFTIdentifierMode  | u8 |
  // ---------------------------
  // | Ordinal            | 0  |
  // ---------------------------
  // | Hash               | 1  |
  // ---------------------------
  //
  // --session-arg "identifier_mode:u8='0'" \
  const identifier_mode = new CLU8(0);
  //
  // --------------------------
  // | MetadataMutability | u8 |
  // ---------------------------
  // | Immutable          | 0  |
  // ---------------------------
  // | Mutable            | 1  |
  // ---------------------------
  //
  // --session-arg "metadata_mutability:u8='0'"
  const metadata_mutability = new CLU8(0); // CAN CHANGE METADATA

  // INSERT MORE ARGUMENTS
  // const dto_dev_hash = new CLString("account-hash-55884917f4107a59e8c06557baee7fdada631af6d1c105984d196a84562854eb") //ABB
  let devAccountHashByte = Uint8Array.from(
    Buffer.from("55884917f4107a59e8c06557baee7fdada631af6d1c105984d196a84562854eb", 'hex'), //ABB
  )

  const csp_dev = createRecipientAddress(new CLAccountHash(devAccountHashByte)) // MPC key 

  //const csp_mint_fee = new CLU64(0);
  // const dto_minter_hash = new CLString("account-hash-55884917f4107a59e8c06557baee7fdada631af6d1c105984d196a84562854eb");
  //const dto_minter1 = new CLString("account-hash-69b994ec6f871de00f099de1f7bcfca61bec1a1699d85ec50e7b883965bbc485"); // MPC 
  let minterAccountHashByte = Uint8Array.from(
    Buffer.from("55884917f4107a59e8c06557baee7fdada631af6d1c105984d196a84562854eb", 'hex'), //MPCkey
  )

  // let minterAccountHashByte = Uint8Array.from(
  //   Buffer.from("55884917f4107a59e8c06557baee7fdada631af6d1c105984d196a84562854eb", 'hex'), //MPCkey
  // )


  const csp_minter = createRecipientAddress(new CLAccountHash(minterAccountHashByte)) // MPC key 


  let feeReceiverAccountHashByte = Uint8Array.from(
    Buffer.from("3355c1d284398a7aa4ac2657aa484d8591c43deba5ae965542f0595f86dd3d52", 'hex'), //MPCkey
  )

  // let minterAccountHashByte = Uint8Array.from(
  //   Buffer.from("55884917f4107a59e8c06557baee7fdada631af6d1c105984d196a84562854eb", 'hex'), //MPCkey
  // )


  const fee_receiver = createRecipientAddress(new CLAccountHash(feeReceiverAccountHashByte)) // MPC key 



  // EXP package
  let exp = "d388c617becf8c00a193b65996b24745761c913abdbc167318d8facf4d7954ba"
  const contracthashbytearray = new CLByteArray(Uint8Array.from(Buffer.from(exp, 'hex')));
  const EXPContractHash = new CLKey(contracthashbytearray);


  //
  // EXP contract
  let contract_minter = "6b85c486ab35bff046ac03d4558639b62fea3db0cfe1153eeb88bd1faca6f20e"
  const contractExpbytearray = new CLByteArray(Uint8Array.from(Buffer.from(contract_minter, 'hex')));
  const MinterContract = new CLKey(contractExpbytearray);

  // Fee

  let pathWasm = `./contract/target/wasm32-unknown-unknown/release/contract.wasm`;


  const runtimeArgs = RuntimeArgs.fromMap({
    the_contract_owner: csp_dev,
    collection_name: collection_name,
    collection_symbol: collection_symbol,
    total_token_supply: total_token_supply,
    ownership_mode: ownership_mode,
    nft_kind: nft_kind,
    minting_mode: minting_mode,
    holder_mode,
    json_schema,
    allow_minting,
    nft_metadata_kind,
    identifier_mode,
    metadata_mutability,
    owner_reverse_lookup_mode,
    events_mode: events_mode,
    the_contract_minter: MinterContract,
  })
  console.log(CHAIN_NAME,
    NODE_ADDRESS,
    KEYS,
    runtimeArgs,
    paymentAmount,
    WASM_PATH
  )
  // let newPath = "./scripts/contract.wasm"
  // let pathWasmOfficial = `../cep78New/cep-78-enhanced-nft/contract/target/wasm32-unknown-unknown/release/contract.wasm`;
  console.log("DDDDD")
  let hash = await installContract(
    CHAIN_NAME,
    NODE_ADDRESS,
    KEYS,
    runtimeArgs,
    paymentAmount,
    WASM_PATH
  );
  console.log("B")

  console.log(`... Contract installation deployHash: ${hash}`)

  await getDeploy(NODE_ADDRESS, hash)

  let accountInfo = await utils.getAccountInfo(NODE_ADDRESS, KEYS.publicKey)

  console.log(`... Contract installed successfully.`)

  console.log(`... Account Info: `)
  console.log(JSON.stringify(accountInfo, null, 2))
  fs.writeFileSync('scripts/contractinfo.json', JSON.stringify(accountInfo, null, 2));


};

main();