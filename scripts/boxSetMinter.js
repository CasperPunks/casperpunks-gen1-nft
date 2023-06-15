require("dotenv").config();
const { CasperContractClient, helpers } = require("casper-js-client-helper");
const { getDeploy, getOperatorDictionaryKey } = require("./indexC");
const { createRecipientAddress } = helpers;
const CEP78 = require('./box-cep78.js')
let key = require('./keys.json').key
const sha256 = require("js-sha256")

const { CLValueBuilder, Keys, RuntimeArgs, CLByteArrayBytesParser, CLByteArray, CLKey, CLPublicKey, CLAccountHash } = require("casper-js-sdk");

const { NODE_ADDRESS, EVENT_STREAM_ADDRESS, CHAIN_NAME, WASM_PATH } =
  process.env;

let privateKeyPem = `
-----BEGIN PRIVATE KEY-----
${key}
-----END PRIVATE KEY-----
`; // abb key
let contractHash = "64aabeaa53ada9eaa1265774ecd1f28de052c9f059eb2b2511e1a99ea022f097" // wrap 721
//let contractHash = "97ec1fdd4281b3ea73039f749fc784d80c3a7c562eba5a6a9adca223e3b5aca2"
let toAddress = "020261207299a7d59261d28a0780b92f76b5caff3ee2e3f767d7cd832e269c181767" // publicKey


let privateKeyBuffer = Keys.Ed25519.parsePrivateKey(
  Keys.Ed25519.readBase64WithPEM(privateKeyPem)
);
let publicKey = Keys.Ed25519.privateToPublicKey(
  Uint8Array.from(privateKeyBuffer)
);
let KEYS = new Keys.Ed25519.parseKeyPair(
  publicKey,
  Uint8Array.from(privateKeyBuffer)
);

async function main() {
  console.log("B", NODE_ADDRESS, CHAIN_NAME)
  let csp = await CEP78.createInstance(contractHash, NODE_ADDRESS, CHAIN_NAME)

  console.log("A")
  let factoryPackage = "76b9625447de65b85924bb697c95bda151b7dad363915047f11bfeb15d4a7c19"

  try {

    let hash = await csp.setMinter({
      keys: KEYS,
      minter: factoryPackage,
    })

    console.log(`... Contract installation deployHash: ${hash}`);

    await getDeploy(NODE_ADDRESS, hash);

    console.log(`... Contract installed successfully.`);



  } catch (e) {
    console.error(e)
  }
}

main();
