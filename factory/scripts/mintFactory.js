require("dotenv").config();
let contractInfo = require("./contractinfo.json");
const { CasperContractClient, helpers } = require("casper-js-client-helper");
const { getDeploy } = require("./utils");
const { createRecipientAddress } = helpers;
const sdk = require('../../indexCasperPunk')
let key = require('./keys.json').key
let keytonya = require('./keys.json').keyTonya
let keytonyb = require('./keys.json').keyTonyb
let keytonyc = require('./keys.json').keyTonyc
let keyPhuong = require('./keys.json').keyPhuong


const { CLValueBuilder, Keys, RuntimeArgs, CLByteArrayBytesParser, CLByteArray, CLKey, CLPublicKey, CLAccountHash } = require("casper-js-sdk");

const { NODE_ADDRESS, EVENT_STREAM_ADDRESS, CHAIN_NAME, WASM_PATH } =
  process.env;

let privateKeyPem = `
-----BEGIN PRIVATE KEY-----
${key}
-----END PRIVATE KEY-----
`; // abb key



let privateKeyPemP = `
-----BEGIN PRIVATE KEY-----
${keyPhuong}
-----END PRIVATE KEY-----
`; // phuong key



let privateKeyPemTony = `
-----BEGIN PRIVATE KEY-----
${keytonya}
${keytonyb}
${keytonyc}
-----END PRIVATE KEY-----
`; // tony key


// let factoryHash = contractInfo.namedKeys
//   .filter((e) => e.name == "csp_factory_contract")[0]
//   .key.slice(5);
let factoryHash = "60cd90c9196ee34f44bb9faff9d7dce0563368b37637afbcc82279f3c1ec8046"
console.log("csp_factory_contract: ", factoryHash)
let nft_contract = 
// "39a2c626a00415332171109def12a06be37e5f109b234be355afaf86a63046f3" // CSP package hash
"39069af35e63318419bb02ef75abc5782e89c0d3ac57319fb62ec6e12198b6dd" // CSP package hash

//"a7643ef321cce2cd1401a338be87c1a6cffffe4f482b5364f35ccc1f085e9c22" // CSP contract
//  "6fcf59753e5ab985122a88470101acb338594614266a506a2e3cf57025bc4ddc"
// "68d05b72593981f73f5ce7ce5dcac9033aa0ad4e8c93b773f8b939a18c0bbc3b";
//"805347b595cc24814f0d50482069a1dba24f9bfb2823c6e900386f147f25754b";
//"52f370db3aeaa8c094e73a3aa581c85abc775cc52605e9cd9364cae0501ce645";
//"44f244fb474431a20c4968d60550f790000d21785650c963f9ac5e02c126e1fb";

let toAddress = "020261207299a7d59261d28a0780b92f76b5caff3ee2e3f767d7cd832e269c181767" // publicKey

// Key phuong

let privateKeyBufferP = Keys.Ed25519.parsePrivateKey(
  Keys.Ed25519.readBase64WithPEM(privateKeyPemP)
);
let publicKeyP = Keys.Ed25519.privateToPublicKey(
  Uint8Array.from(privateKeyBufferP)
);
let KEYSP = new Keys.Ed25519.parseKeyPair(
  publicKeyP,
  Uint8Array.from(privateKeyBufferP)
);

// Key 

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
  let factory = await sdk.CSPFactory.createInstance(factoryHash, NODE_ADDRESS, CHAIN_NAME)
  // let cep78 = await sdk.CEP78.createInstance(nft_contract, NODE_ADDRESS, CHAIN_NAME)
  // const contracthashbytearray = new CLByteArray(Uint8Array.from(Buffer.from("5db43d7bda61a954f4a73d51de9ee3a1c1a58d2b9cf895e1b98c6d3f73ee38e9", 'hex')));
  // const nftContractHash = new CLKey(contracthashbytearray);

  // let hashApprove = await cep78.approveForAll({
  //   keys: KEYS,
  //   operator: nftContractHash
  // })
  // console.log(`... Contract installation deployHash: ${hashApprove}`);

  // await getDeploy(NODE_ADDRESS, hashApprove);

  const meta_data_json = {
    "name": "Casper Punk",
    "symbol": "CSP",
    "token_uri": "ipfs://QmeSjSinHpPnmXmspMjwiXyN6zS4E9zccariGR3jxcaWtq/21",
    "checksum": "940bffb3f2bba35f84313aa26da09ece3ad47045c6a1292c2bbd2df4ab1a55fb",
    "rarity" : 0,
    "stamina" : 0,
    "charisma": 0,
    "intelligence" : 0,
  }



  let hash = await factory.mint({
    keys: KEYSP,
    nftContractHash: nft_contract, // CEP78
    metadata: meta_data_json,
  })

  console.log(`... Contract installation deployHash: ${hash}`);

  await getDeploy(NODE_ADDRESS, hash);

  console.log(`... Contract installed successfully.`);
};

test();
