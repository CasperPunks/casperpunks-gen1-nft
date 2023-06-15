require('dotenv').config()
const fs = require('fs');

const { utils, helpers } = require('casper-js-client-helper')
const { sleep, getDeploy } = require('../../marketplace/scripts/utils')
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
let key = require('../../marketplace/scripts/keys.json').key
let keytonya = require('../../marketplace/scripts/keys.json').keyTonya
let keytonyb = require('../../marketplace/scripts/keys.json').keyTonyb
let keytonyc = require('../../marketplace/scripts/keys.json').keyTonyc
let CEP78 = require("./CSP-cep78");


const { NODE_ADDRESS, EVENT_STREAM_ADDRESS, CHAIN_NAME, WASM_PATH } =
  process.env;

let privateKeyPem = `
-----BEGIN PRIVATE KEY-----
${key}
-----END PRIVATE KEY-----
`; // abb key

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

const test = async () => {
  let nft_contract =
    "97ec1fdd4281b3ea73039f749fc784d80c3a7c562eba5a6a9adca223e3b5aca2" // CSP contract

  let contract = new CEP78(nft_contract, NODE_ADDRESS, CHAIN_NAME);
  await contract.init();
  const contracthashbytearray = new CLByteArray(Uint8Array.from(Buffer.from("c74f5b80205b04861a698e0357c8041b3476c0520809a7a377d0bf2e433bda0b", 'hex'))); // marketplace
  console.log("contracthashbytearray: ", contracthashbytearray)
  // const nftContractHash = new CLKey(contracthashbytearray);

  let hashApprove = await contract.approveForAll({
    keys: KEYSTony,
    operator: contracthashbytearray
  })
  console.log(`... Contract installation deployHash: ${hashApprove}`);

  await getDeploy(NODE_ADDRESS, hashApprove);


  console.log(`... Contract installation deployHash: ${hashApprove}`);

  console.log(`... Contract installed successfully.`);
};

test();
