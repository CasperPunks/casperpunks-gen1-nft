require("dotenv").config();
const { CLAccountHash, CLPublicKey, U64_ID, U256_ID } = require("casper-js-sdk");
let CEP78 = require("./CSP-cep78");
let contractHash =
  //"f22f266ea7171e2e3c7c21266c8b7f0da2ddee8b2357ca85339af73a4018d374";
  //"fd7427f382e6129faf66a886e77bfdcc6edfd183813aab1b0b553cbd96a9c500";
  //"d2605b4544842bced2eeed6d3c541e220cd4942ad39cbd5e3ea14d475cdf194b"; // new 43113 wrapped token
  //"805347b595cc24814f0d50482069a1dba24f9bfb2823c6e900386f147f25754b";
  //"52f370db3aeaa8c094e73a3aa581c85abc775cc52605e9cd9364cae0501ce645";
  //"68d05b72593981f73f5ce7ce5dcac9033aa0ad4e8c93b773f8b939a18c0bbc3b";
  "be170557d9100704b63dc5de6039373dfdc8649cc467bf2f74ea14812d233def";
let contractInfo = require("./contractinfo.json");
let nft_bridge_contract = contractInfo.namedKeys
  .filter((e) => e.name == "dotoracle_nft_bridge_contract")[0]
  .key.slice(5);
const { NODE_ADDRESS, EVENT_STREAM_ADDRESS, CHAIN_NAME, WASM_PATH } =
  process.env;
async function main() {
  let contract = new CEP78(contractHash, NODE_ADDRESS, CHAIN_NAME);
  await contract.init();
  // let identifier_mode = await contract.identifierMode();
  // console.log("identifier_mode", identifier_mode.toString());

  //   let ownerOf = await cep78.getOwnerOf(31);
  //   console.log("ownerOf", ownerOf);

  //   let balanceOf = await cep78.balanceOf("3bdcc50ce1e1e0119d4901b686c65c66b63cc17e5fa5da2299e332c545ec23c6")
  //   console.log('balanceOf', balanceOf.toString())

  //   let burntTokens = await cep78.burntTokens(31);
  //   console.log("burntTokens", burntTokens);

  //   let metadata = await cep78.getTokenMetadata(31);
  //   console.log("metadata", metadata);

  //   let operator = await cep78.getOperator(31);
  //   console.log("operator", operator);

  try {
    let metadata = await contract.getOwnedTokens(CLPublicKey.fromHex("0131e805fde6a85b63aa366990136b4759a596d9a988bde62b84131bc86a910e6b")) // tony account
    let meta1 = await contract.getTokenMetadata(0)
    //let metadata = await contract.getOwnedTokens(CLPublicKey.fromHex("0121eb7d280926cd62ae0b44ee628ba057e9b2696021ab0e20e40e528ae243bde1"))// Vi hka
    let bal = await contract.balanceOf(CLPublicKey.fromHex("0131e805fde6a85b63aa366990136b4759a596d9a988bde62b84131bc86a910e6b"))
    // let bal = await contract.balanceOf(CLPublicKey.fromHex("020261207299a7d59261d28a0780b92f76b5caff3ee2e3f767d7cd832e269c181767"))

    console.log("metadata", metadata.map((e)=> parseInt(e)));
    console.log("bal: ", parseInt(bal))
    console.log("meta1: ", meta1)
  } catch (e) {
    console.error(e)
  }
}

main();
