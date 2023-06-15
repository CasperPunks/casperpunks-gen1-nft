require('dotenv').config()
const fs = require('fs');

const { utils, helpers } = require('casper-js-client-helper')
const { sleep, getDeploy } = require('./utils')

const {
  CLValueBuilder,
  Keys,
  CLPublicKey,
  CLPublicKeyType,
  RuntimeArgs,
  CLString,
  CLAccountHash,
  CLByteArray
} = require('casper-js-sdk')
let key = require('./keys.json').key

const {
  fromCLMap,
  toCLMap,
  installContract,
  setClient,
  contractSimpleGetter,
  contractCallFn,
  createRecipientAddress
} = helpers;

const {
  NODE_ADDRESS,
  EVENT_STREAM_ADDRESS,
  CHAIN_NAME,
  WASM_PATH
} = process.env
let paymentAmount = '180000000000' //140

let privateKeyPem = `
-----BEGIN PRIVATE KEY-----
${key}
-----END PRIVATE KEY-----
`

let privateKeyBuffer = Keys.Ed25519.parsePrivateKey(Keys.Ed25519.readBase64WithPEM(privateKeyPem))
let publicKey = Keys.Ed25519.privateToPublicKey(Uint8Array.from(privateKeyBuffer))
let KEYS = new Keys.Ed25519.parseKeyPair(publicKey, Uint8Array.from(privateKeyBuffer))
console.log('pubkey', KEYS.accountHex())
let contract_key_name = "box_factory"
let dev = "017e80955a6d493a4a4b9f1b5dd23d2edcdc2c8b00fcd9689f2f735f501bd088c5" // ABB
let feeReceiver = "0161e83e521241390a555224f3455c192712f9c74eb776663ed4a909a9c87efffc"

let boxPKHash = "27331e391e05f201a57e40ed18ea68f5438e0a5d50800de2293e0f29429be6e4"
boxPKHash = new CLByteArray(
  Uint8Array.from(Buffer.from(boxPKHash, "hex"))
);
let boxKey = createRecipientAddress(boxPKHash)

const test = async () => {

  const runtimeArgs = RuntimeArgs.fromMap({
    "csp_factory_contract": CLValueBuilder.string(contract_key_name),
    "contract_owner": createRecipientAddress(CLPublicKey.fromHex(dev)), //ABB
    "dev": createRecipientAddress(CLPublicKey.fromHex(dev)), // ABB
    "fee_receiver": createRecipientAddress(CLPublicKey.fromHex(feeReceiver)), // ABB
    "mint_fee": CLValueBuilder.u256("3000000000000"), // 8 cspr
    "total_box": CLValueBuilder.u64("1000"),
    "max_per_one": CLValueBuilder.u8("10"),
    "minting_start_time": CLValueBuilder.u64("1681300727"),
    "minting_end_time": CLValueBuilder.u64(1681300727 + 1800),
    "mint_fee_r3": CLValueBuilder.u256("4000000000000"), // 8 cspr
    "total_box_r3": CLValueBuilder.u64("4000"),
    "max_per_one_r3": CLValueBuilder.u8("25"),
    "minting_start_time_r3": CLValueBuilder.u64(1681300727 + 900),
    "nft_contract_package": boxKey

  });

  console.log("A")

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

  // const contractHash = await utils.getAccountNamedKeyValue(
  //   accountInfo,
  //   `erc20_token_contract`,
  // )

  // await getDeploy(NODE_ADDRESS!, installDeployHash)

  // console.log(`... Contract installed successfully.`)

  // let accountInfo = await utils.getAccountInfo(NODE_ADDRESS!, KEYS.publicKey)

  // console.log(`... Account Info: `)
  // console.log(JSON.stringify(accountInfo, null, 2))

  // const contractHash = await utils.getAccountNamedKeyValue(
  //   accountInfo,
  //   `erc20_token_contract`,
  // )

  // await erc20.setContractHash(
  //   contractHash.slice(
  //     5
  //   )
  // );

  // console.log(`... Contract Hash: ${contractHash}`)

  // let deployed_minter = await erc20.minter()
  // console.log(`... deployed_minter: ${deployed_minter}`)
  // console.log(`... fee: ${await erc20.swapFee()}`)
  // console.log(`... dev: ${await erc20.dev()}`)
}

test()
