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
let contractHash = "bcbfa6148e89086a0c3664e7f531dc41ac08ff7343447b88aa304092a91b22f0" // wrap 721
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

  const meta_data_json = {
    "name": "Mistery Box",
    "symbol": "MBOX",
    "token_uri": "https://api-gen0.casperpunks.io/lootbox.png",
    "checksum": sha256("https://api-gen0.casperpunks.io/lootbox.png")
    // "stamina": 0,
    // "charisma": 0,
    // "intelligence": 0,
    // "rarity": 0,
  }

  let account1 = "017e80955a6d493a4a4b9f1b5dd23d2edcdc2c8b00fcd9689f2f735f501bd088c5" // account hash
  let account2 = "020261207299a7d59261d28a0780b92f76b5caff3ee2e3f767d7cd832e269c181767" // account hash

  // account1 = "55884917f4107a59e8c06557baee7fdada631af6d1c105984d196a84562854eb"
  console.log("A")

  try {
    for (var i = 0; i < 1; i++) {

      let hash = await csp.mintBoxx({
        keys: KEYS,
        metadataJson: meta_data_json,
        tokenOwners: ["017e80955a6d493a4a4b9f1b5dd23d2edcdc2c8b00fcd9689f2f735f501bd088c5", "020261207299a7d59261d28a0780b92f76b5caff3ee2e3f767d7cd832e269c181767", "0169891ce0333a946f8fba84f83e9f9173244af6d71ca9a30b24bc25ce8610aa77", "0131e805fde6a85b63aa366990136b4759a596d9a988bde62b84131bc86a910e6b"],
        numberOfBoxs: [10, 10, 10, 10],
      })

      console.log(`... Contract installation deployHash: ${hash}`);

      await getDeploy(NODE_ADDRESS, hash);

      console.log(`... Contract installed successfully.`);

    }

  } catch (e) {
    console.error(e)
  }
}

main();
