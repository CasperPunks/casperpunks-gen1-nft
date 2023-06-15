require('dotenv').config()
const fs = require('fs');

const { utils, helpers } = require('casper-js-client-helper')
const { getDeploy, getOperatorDictionaryKey } = require("./indexC");

// const { genRanHex } = require("../indexCasperPunk")
const sha256 = require("js-sha256")
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
  //PAYMENT_WASM_PATH,
} = process.env
let paymentAmount = '38000000000' //3

let privateKeyPem = `
-----BEGIN PRIVATE KEY-----
${key}
-----END PRIVATE KEY-----
`

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



let privateKeyBuffer = Keys.Ed25519.parsePrivateKey(Keys.Ed25519.readBase64WithPEM(privateKeyPem))
let publicKey = Keys.Ed25519.privateToPublicKey(Uint8Array.from(privateKeyBuffer))
let KEYS = new Keys.Ed25519.parseKeyPair(publicKey, Uint8Array.from(privateKeyBuffer))
console.log('pubkey', KEYS.accountHex())
let contract_key_name = "csp_factory_contract"
let contract_owner = "02038df1cff6b55615858b1acd2ebcce98db164f88cf88919c7b045268571cc49cb7" // MPC
let dev = "017e80955a6d493a4a4b9f1b5dd23d2edcdc2c8b00fcd9689f2f735f501bd088c5" // ABB
const test = async () => {


  let fac = "63ed815e9df9eee6020f5bf958633ab11a1c567501aae043f781b7e8683ed9cd"
  fac = new CLByteArray(
    Uint8Array.from(Buffer.from(fac, "hex"))
  );
  let facKey = createRecipientAddress(fac)



  let boxPKHash = "c2896cb54c296e4177c081a29492f3a0f7beb7ad4392d1aa6eae09aca3f5d005"
  boxPKHash = new CLByteArray(
    Uint8Array.from(Buffer.from(boxPKHash, "hex"))
  );
  let boxPK = createRecipientAddress(boxPKHash)


  // let nftContractHash = "58bf9d46ff7e432ab4c4b1f53ba759fc49f7b235c23bb851babac0561c6210b1" // CSP hash
  // console.log("nftContractHash: ", nftContractHash)
  // nftContractHash = nftContractHash.startsWith("hash-")
  //   ? nftContractHash.slice(5)
  //   : nftContractHash;
  // nftContractHash = new CLByteArray(
  //   Uint8Array.from(Buffer.from(nftContractHash, "hex"))
  // );
  // let nftCep47Hash = createRecipientAddress(nftContractHash)

  const meta_data_json = {
    "name": "Mystery Box",
    "symbol": "MBOX",
    "token_uri": "https://api-gen0.casperpunks.io/lootbox.png",
    "checksum": sha256("https://api-gen0.casperpunks.io/lootbox.png")
  }
  let token_metadata = new CLString(JSON.stringify(meta_data_json))



  let runtimeArgs = RuntimeArgs.fromMap({
    "deposit_entry_point_name": CLValueBuilder.string("mint"),
    "amount": CLValueBuilder.u512("100000000000"), // 8 cspr
    "factory_contract_hash": facKey,
    "nft_contract_package": boxPK,
    "count": CLValueBuilder.u8(1),
    "token_metadata": token_metadata,
    "token_owner": createRecipientAddress(CLPublicKey.fromHex("020261207299a7d59261d28a0780b92f76b5caff3ee2e3f767d7cd832e269c181767")), // 1346,8
  })

  // console.log("A")
  // console.log(CHAIN_NAME)
  // console.log(NODE_ADDRESS)
  // console.log(KEYS)
  // console.log(runtimeArgs)
  // console.log(paymentAmount)
  // // console.log(WASM_PATH)
  // console.log("PAYMENT_WASM_PATH: ", PAYMENT_WASM_PATH)

  // let x = utils.getBinary(PAYMENT_WASM_PATH)
  // console.log(Buffer.from(x).toString("hex"))

  let PAYMENT_WASM_PATH = "./scripts/payment_contract.wasm"
  let hash = await installContract(
    CHAIN_NAME,
    NODE_ADDRESS,
    KEYSTony,
    runtimeArgs,
    paymentAmount,
    PAYMENT_WASM_PATH
  );
  console.log("B")

  console.log(`... Contract installation deployHash: ${hash}`)

  await getDeploy(NODE_ADDRESS, hash)
  console.log(`... Contract installed successfully.`)

}

test()
