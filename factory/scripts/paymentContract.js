require('dotenv').config()
const fs = require('fs');

const { utils, helpers } = require('casper-js-client-helper')
const { sleep, getDeploy } = require('./utils')
const { genRanHex } = require("../indexCasperPunk")

const {
  CLValueBuilder,
  Keys,
  CLPublicKey,
  CLPublicKeyType,
  RuntimeArgs,
  CLString,
  CLByteArray,
  CLAccountHash
} = require('casper-js-sdk')
let key = require('./keys.json').key
let keytonya = require('./keys.json').keyTonya
let keytonyb = require('./keys.json').keyTonyb
let keytonyc = require('./keys.json').keyTonyc

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
  WASM_PATH,
  PAYMENT_WASM_PATH,
} = process.env
let paymentAmount = '10000000000' //3

let privateKeyPem = `
-----BEGIN PRIVATE KEY-----
${key}
-----END PRIVATE KEY-----
`

let privateKeyBuffer = Keys.Ed25519.parsePrivateKey(Keys.Ed25519.readBase64WithPEM(privateKeyPem))
let publicKey = Keys.Ed25519.privateToPublicKey(Uint8Array.from(privateKeyBuffer))
let KEYS = new Keys.Ed25519.parseKeyPair(publicKey, Uint8Array.from(privateKeyBuffer))
console.log('pubkey', KEYS.accountHex())

let privateKeyPemTony = `
-----BEGIN PRIVATE KEY-----
${keytonya}
${keytonyb}
${keytonyc}
-----END PRIVATE KEY-----
`; // tony key
let privateKeyBufferTony = Keys.Secp256K1.parsePrivateKey(
  Keys.Secp256K1.readBase64WithPEM(privateKeyPemTony)
);


let publicKeyTony = Keys.Secp256K1.privateToPublicKey(
  Uint8Array.from(privateKeyBufferTony)
);

let KEYSTony = new Keys.Secp256K1.parseKeyPair(
  publicKeyTony,
  Uint8Array.from(privateKeyBufferTony),
  "raw"
);




let contract_key_name = "csp_factory_contract"
let contract_owner = "02038df1cff6b55615858b1acd2ebcce98db164f88cf88919c7b045268571cc49cb7" // MPC
let dev = "017e80955a6d493a4a4b9f1b5dd23d2edcdc2c8b00fcd9689f2f735f501bd088c5" // ABB
const test = async () => {

  const meta_data_json = {
    "name": "Casper Punk",
    "symbol": "CSP",
    "token_uri": "ipfs://QmeSjSinHpPnmXmspMjwiXyN6zS4E9zccariGR3jxcaWtq/12",
    "checksum": "940bffb3f2bba35f84313aa26da09ece3ad47045c6a1292c2bbd2df4ab1a55fb",
    "rarity": 0,
    "stamina": 0,
    "charisma": 0,
    "intelligence": 0,
  }
  const token_meta_data = new CLString(JSON.stringify(meta_data_json))

  let requestId = CLValueBuilder.string(genRanHex())
  console.log("RequestId: ", requestId)

  let factoryHash = "56dc09037d01edaee4d166035b176dc07a6697a3b93ce83f1fe8a211045125fd"
  console.log("factory package: ", factoryHash)
  factoryHash = factoryHash.startsWith("hash-")
    ? factoryHash.slice(5)
    : factoryHash;
  factoryHash = new CLByteArray(
    Uint8Array.from(Buffer.from(factoryHash, "hex"))
  );
  let factoryKey = createRecipientAddress(factoryHash)


  let nftContractHash = "39a2c626a00415332171109def12a06be37e5f109b234be355afaf86a63046f3" // CSP hash
  console.log("nftContractHash: ", nftContractHash)
  nftContractHash = nftContractHash.startsWith("hash-")
    ? nftContractHash.slice(5)
    : nftContractHash;
  nftContractHash = new CLByteArray(
    Uint8Array.from(Buffer.from(nftContractHash, "hex"))
  );
  let nftCep78Hash = createRecipientAddress(nftContractHash)



  let runtimeArgs = RuntimeArgs.fromMap({
    "factory_package": factoryKey, 
    "nft_contract_package": nftCep78Hash,
    "token_meta_data": token_meta_data,
    "request_id": requestId,
    "amount": CLValueBuilder.u512("9000000000"), // 8 cspr
    "deposit_entry_point_name": new CLString("mint")
  })



  console.log("A")
  // console.log(CHAIN_NAME)
  // console.log(NODE_ADDRESS)
  // console.log(KEYS)
  // console.log(runtimeArgs)
  // console.log(paymentAmount)
  // console.log(WASM_PATH)
  console.log("PAYMENT_WASM_PATH: ", PAYMENT_WASM_PATH)

  let hash = await installContract(
    CHAIN_NAME,
    NODE_ADDRESS,
    //KEYS,
    KEYSTony,
    runtimeArgs,
    paymentAmount,
    PAYMENT_WASM_PATH
  );
  console.log("B")

  console.log(`... Contract installation deployHash: ${hash}`)

  await getDeploy(NODE_ADDRESS, hash)

  let accountInfo = await utils.getAccountInfo(NODE_ADDRESS, KEYS.publicKey)

  console.log(`... Contract installed successfully.`)

  // console.log(`... Account Info: `)
  // console.log(JSON.stringify(accountInfo, null, 2))
  // fs.writeFileSync('scripts/contractinfo.json', JSON.stringify(accountInfo, null, 2));

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
