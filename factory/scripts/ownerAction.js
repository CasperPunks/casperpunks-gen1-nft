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


const { CLValueBuilder, Keys, RuntimeArgs, CLByteArrayBytesParser, CLByteArray, CLKey, CLPublicKey, CLAccountHash } = require("casper-js-sdk");

const { NODE_ADDRESS, EVENT_STREAM_ADDRESS, CHAIN_NAME, WASM_PATH } =
  process.env;

let privateKeyPem = `
-----BEGIN PRIVATE KEY-----
${key}
-----END PRIVATE KEY-----
`; // abb key
let privateKeyPemTony = `
-----BEGIN PRIVATE KEY-----
${keytonya}
${keytonyb}
${keytonyc}
-----END PRIVATE KEY-----
`; // tony key


let newOwnerPub =
  "020261207299a7d59261d28a0780b92f76b5caff3ee2e3f767d7cd832e269c181767"
// "805347b595cc24814f0d50482069a1dba24f9bfb2823c6e900386f147f25754b"
let oldOwnerPub = "017e80955a6d493a4a4b9f1b5dd23d2edcdc2c8b00fcd9689f2f735f501bd088c5"
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

let factoryHash = "61752f80e347c8f0331a03af9b5b955fced5e6a97d98467d0a1036a97d12cd4b"


const test = async () => {


  let factory = await sdk.CSPFactory.createInstance(factoryHash, NODE_ADDRESS, CHAIN_NAME)


  // transfer owner

  let hashTransferOwner = await factory.transferOwner({
    keys: KEYS,
    newOwner: newOwnerPub,
  })

  console.log(`... hashTransferOwner installation deployHash: ${hashTransferOwner}`);

  await getDeploy(NODE_ADDRESS, hashTransferOwner);

  console.log(`... hashTransferOwner installed successfully.`);


  // Transfer owner back

  let hashTransferOwner1 = await factory.transferOwner({
    keys: KEYSTony,
    newOwner: oldOwnerPub,
  })

  console.log(`... Transfer owner back installation deployHash: ${hashTransferOwner1}`);

  await getDeploy(NODE_ADDRESS, hashTransferOwner1);

  console.log(`... Transfer owner back installed successfully.`);


  // Change WCSPR contract
  let newWcspr = "805347b595cc24814f0d50482069a1dba24f9bfb2823c6e900386f147f25754b"

  let hashChangeWcspr = await factory.changeWcsprContract({
    keys: KEYS,
    newWcsprContract: newWcspr,
  })

  console.log(`... hashChangeWcspr installation deployHash: ${hashChangeWcspr}`);

  await getDeploy(NODE_ADDRESS, hashChangeWcspr);

  console.log(`... hashChangeWcspr installed successfully.`);

  // Change WCSPR contract back
  let newWcsprback = "30070685c86e7fb410839f1ffc86de2181d4776926248e0946350615929b1ce2"

  let hashChangeWcsprback = await factory.changeWcsprContract({
    keys: KEYS,
    newWcsprContract: newWcsprback,
  })

  console.log(`... hashChangeWcsprback installation deployHash: ${hashChangeWcsprback}`);

  await getDeploy(NODE_ADDRESS, hashChangeWcsprback);

  console.log(`... hashChangeWcsprback installed successfully.`);



  // Change Fee receiver 
  let newReceiver = "020261207299a7d59261d28a0780b92f76b5caff3ee2e3f767d7cd832e269c181767"

  let hashChangeWFeeReceiver = await factory.changeFeeReceiver({
    keys: KEYS,
    newReceiver: newReceiver,
  })

  console.log(`... hashChangeWFeeReceiver installation deployHash: ${hashChangeWFeeReceiver}`);

  await getDeploy(NODE_ADDRESS, hashChangeWFeeReceiver);

  console.log(`... hashChangeWFeeReceiver installed successfully.`);

    // Change Fee receiver back
    let newReceiverback = "017e80955a6d493a4a4b9f1b5dd23d2edcdc2c8b00fcd9689f2f735f501bd088c5"

    let hashChangeFeeReceiverback = await factory.changeFeeReceiver({
      keys: KEYS,
      newReceiver: newReceiverback,
    })
  
    console.log(`... hashChangeFeeReceiverback installation deployHash: ${hashChangeFeeReceiverback}`);
  
    await getDeploy(NODE_ADDRESS, hashChangeFeeReceiverback);
  
    console.log(`... hashChangeFeeReceiverback installed successfully.`);
  
  


  // Change mint fee 

  let hashChangeFee = await factory.changeMintFee({
    keys: KEYS,
    newFee: "10000000000",
  })

  console.log(`... hashChangeFee installation deployHash: ${hashChangeFee}`);

  await getDeploy(NODE_ADDRESS, hashChangeFee);

  console.log(`... hashChangeFee installed successfully.`);


};

test();
