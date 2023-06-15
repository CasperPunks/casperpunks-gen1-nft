require("dotenv").config();
let contractInfo = require("./contractinfo.json");
const { CasperContractClient, helpers } = require("casper-js-client-helper");
const { getDeploy } = require("./utils");
const { createRecipientAddress } = helpers;
const sdk = require('./indexCasperPunk')
let key = require('./keys.json').key

const { CLValueBuilder, Keys, RuntimeArgs, CLByteArrayBytesParser, CLByteArray, CLKey, CLPublicKey, CLAccountHash } = require("casper-js-sdk");

const { NODE_ADDRESS, EVENT_STREAM_ADDRESS, CHAIN_NAME, WASM_PATH } =
  process.env;

let privateKeyPem = `
-----BEGIN PRIVATE KEY-----
${key}
-----END PRIVATE KEY-----
`; // abb key

// let factoryHash = contractInfo.namedKeys
//   .filter((e) => e.name == "csp_factory_contract")[0]
//   .key.slice(5);

let factoryHash = "6c0527bec8f9c53c1bfc5b14285f81218c0091f6d6f500821aae186f996af5eb"
console.log("csp_factory_contract: ", factoryHash)

let nft_contract =
  "6fcf59753e5ab985122a88470101acb338594614266a506a2e3cf57025bc4ddc"

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

const test = async () => {
  let factory = await sdk.CSPFactory.createInstance(factoryHash, NODE_ADDRESS, CHAIN_NAME)


  let hashSetWL = await factory.setAddressesWhitelist({
    keys: KEYS,
    addressesWhitelistArray: [
      "0169891ce0333a946f8fba84f83e9f9173244af6d71ca9a30b24bc25ce8610aa77",
      "020261207299a7d59261d28a0780b92f76b5caff3ee2e3f767d7cd832e269c181767"
    ],
  })

  console.log(`... hashSetWL installation deployHash: ${hashSetWL}`);

  await getDeploy(NODE_ADDRESS, hashSetWL);

  console.log(`... hashSetWL installed successfully.`);



  // // update
  // let hashUpdate = await factory.updateAddressesWhitelist({
  //   keys: KEYS,
  //   addressesWhitelistArray: [
  //     "017e80955a6d493a4a4b9f1b5dd23d2edcdc2c8b00fcd9689f2f735f501bd088c5",
  //     "01635d0d7306689264017b74818cbe1124027757e06f10956128e6006d5b4d2a36",
  //     "020261207299a7d59261d28a0780b92f76b5caff3ee2e3f767d7cd832e269c181767",
  //     "012fdc613e28b425e3bce46ac5db83ea5782ad1ca9d9f57da7f7982622c862e111",
  //   ],
  //   numberOfTickets : 120,
  // })

  // console.log(`... hashUpdate installation deployHash: ${hashUpdate}`);

  // await getDeploy(NODE_ADDRESS, hashUpdate);

  // console.log(`... hashUpdate installed successfully.`);



};

test();
