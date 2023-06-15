import {
  DeployUtil,
  CasperClient,
  RuntimeArgs,
  CLString,
  CLU64,
  CLU8,
  CLU256,
  CLOption,
  CLBool,
  CLAccountHash,
  CLByteArray,
  CLKey,
  CLValueBuilder,
} from "casper-js-sdk";
import * as utils from "./utils.js";
import * as constants from "./constants";
import { Some } from "ts-results";
import * as Utils from "./indexC";
const {
  helpers,
  CasperContractClient,
} = require("casper-js-client-helper");
const { setClient, contractSimpleGetter, createRecipientAddress } = helpers;


const main = async () => {
  //Step 1: Set casper node client
  const client = new CasperClient(constants.DEPLOY_NODE_ADDRESS);

  //Step 2: Set user key pair

  const pairKeyView = utils.getKeyPairFromPrivateFile(  // ABB
    constants.PATH_TO_PRIVATE_KEYS

  );

  //console.log("pairKeyView : ", pairKeyView)


  //   --session-arg "collection_name:string=''" \
  const collection_name = new CLString("Mistery Box");
  // --session-arg "collection_symbol:string=''" \
  const collection_symbol = new CLString("MBOX");
  // --session-arg "total_token_supply:u64='10'" \
  const total_token_supply = new CLU64("10000");
  // --------------------
  // |    Mode      | u8 |
  // --------------------
  // | Minter       | 0  |
  // --------------------
  // | Assigned     | 1  |
  // --------------------
  // | Transferable | 2  |
  // --------------------
  //
  // --session-arg "ownership_mode:u8='2'" \
  //
  const ownership_mode = new CLU8("2");
  //
  // --------------------
  // |   NFTKind   | u8 |
  // --------------------
  // | Physical    | 0  |
  // --------------------
  // | Digital     | 1  |
  // --------------------
  // | Virtual     | 2  |
  // --------------------
  //
  // --session-arg "nft_kind:u8='1'" \
  //
  const nft_kind = new CLU8(1);
  //
  // --------------------
  // | NFTHolderMode | u8 |
  // --------------------
  // | Accounts      | 0  |
  // --------------------
  // | Contracts     | 1  |
  // --------------------
  // | Mixed         | 2  |
  // --------------------
  //
  // --session-arg "holder_mode:opt_u8='2'" \
  const holder_mode = new CLOption(Some(new CLU8(2)));
  //const holder_mode = new CLU8(2);


  // --session-arg "holder_mode:opt_u8='2'" \
  const events_mode = new CLU8(2) //new CLOption(Some(new CLU8(1)));
  //const holder_mode = new CLU8(2);


  // owner_reverse_lookup_mode

  const owner_reverse_lookup_mode = new CLU8(1) //new CLOption(Some(new CLU8(1)));


  //
  // --------------------
  // | MintingMode | u8 |
  // --------------------
  // | Installer   | 0  |
  // --------------------
  // | Public      | 1  |
  // --------------------
  //
  const minting_mode = new CLU8(1);

  // Optional
  //
  // --session-arg "json_schema:string='nft-schema'" \
  const json_schema = new CLString("nft-schema");
  //
  // allows minting when true
  //
  // --session-arg "allow_minting:bool='true'" \
  //
  const allow_minting = new CLBool(true);
  //
  // --------------------------
  // | NFTMetadataKind  | u8 |
  // -------------------------
  // | CEP78            | 0  |
  // -------------------------
  // | NFT721           | 1  |
  // -------------------------
  // | Raw              | 2  |
  // -------------------------
  // | CustomValidated  | 3  |
  // --------------------------
  // | CasperPunk       | 4  |
  // --------------------------
  //
  // == CEP-78 metadata example
  // {
  // "name": "John Doe",
  // "token_uri": "https://www.barfoo.com",
  // "checksum": "940bffb3f2bba35f84313aa26da09ece3ad47045c6a1292c2bbd2df4ab1a55fb"
  // }
  // ==
  // --session-arg "nft_metadata_kind:u8='1'" \
  //
  const nft_metadata_kind = new CLU8(0);
  //
  // --------------------------
  // | NFTIdentifierMode  | u8 |
  // ---------------------------
  // | Ordinal            | 0  |
  // ---------------------------
  // | Hash               | 1  |
  // ---------------------------
  //
  // --session-arg "identifier_mode:u8='0'" \
  const identifier_mode = new CLU8(0);
  //
  // --------------------------
  // | MetadataMutability | u8 |
  // ---------------------------
  // | Immutable          | 0  |
  // ---------------------------
  // | Mutable            | 1  |
  // ---------------------------
  //
  // --session-arg "metadata_mutability:u8='0'"
  const metadata_mutability = new CLU8(0); // CAN CHANGE METADATA

  // INSERT MORE ARGUMENTS
  // const dto_dev_hash = new CLString("account-hash-55884917f4107a59e8c06557baee7fdada631af6d1c105984d196a84562854eb") //ABB
  let devAccountHashByte = Uint8Array.from(
    Buffer.from("55884917f4107a59e8c06557baee7fdada631af6d1c105984d196a84562854eb", 'hex'), //ABB
  )

  const csp_dev = createRecipientAddress(new CLAccountHash(devAccountHashByte)) // MPC key 

  //const csp_mint_fee = new CLU64(0);
  // const dto_minter_hash = new CLString("account-hash-55884917f4107a59e8c06557baee7fdada631af6d1c105984d196a84562854eb");
  //const dto_minter1 = new CLString("account-hash-69b994ec6f871de00f099de1f7bcfca61bec1a1699d85ec50e7b883965bbc485"); // MPC 
  let minterAccountHashByte = Uint8Array.from(
    Buffer.from("55884917f4107a59e8c06557baee7fdada631af6d1c105984d196a84562854eb", 'hex'), //MPCkey
  )

  // let minterAccountHashByte = Uint8Array.from(
  //   Buffer.from("55884917f4107a59e8c06557baee7fdada631af6d1c105984d196a84562854eb", 'hex'), //MPCkey
  // )


  const csp_minter = createRecipientAddress(new CLAccountHash(minterAccountHashByte)) // MPC key 


  let feeReceiverAccountHashByte = Uint8Array.from(
    Buffer.from("3355c1d284398a7aa4ac2657aa484d8591c43deba5ae965542f0595f86dd3d52", 'hex'), //MPCkey
  )

  // let minterAccountHashByte = Uint8Array.from(
  //   Buffer.from("55884917f4107a59e8c06557baee7fdada631af6d1c105984d196a84562854eb", 'hex'), //MPCkey
  // )


  const fee_receiver = createRecipientAddress(new CLAccountHash(feeReceiverAccountHashByte)) // MPC key 



  // EXP package
  let exp = "d388c617becf8c00a193b65996b24745761c913abdbc167318d8facf4d7954ba"
  const contracthashbytearray = new CLByteArray(Uint8Array.from(Buffer.from(exp, 'hex')));
  const EXPContractHash = new CLKey(contracthashbytearray);


  //
  // EXP contract
  let expContract = "30070685c86e7fb410839f1ffc86de2181d4776926248e0946350615929b1ce2"
  const contractExpbytearray = new CLByteArray(Uint8Array.from(Buffer.from(expContract, 'hex')));
  const EXPContract = new CLKey(contractExpbytearray);

  // Fee

  let pathWasm = `./contract/target/wasm32-unknown-unknown/release/contract.wasm`;
  let newPath = "./scripts/contract.wasm"
  // let pathWasmOfficial = `../cep78New/cep-78-enhanced-nft/contract/target/wasm32-unknown-unknown/release/contract.wasm`;

  let startMint = new CLU64("1680499659");
  let endMint = new CLU64("1690499659");
  let feeCSPR = new CLU256("10000000000");
  console.log(pairKeyView.publicKey,
    "casper-test",
    constants.DEPLOY_GAS_PRICE,
    constants.DEPLOY_TTL_MS)
  console.log(startMint,
    "minting_end_time", endMint,
    "minting_price", feeCSPR,
    "cspr_receiver", fee_receiver,
    "the_contract_owner", csp_dev,
  )
  console.log("DDDDD")
  let deploy = DeployUtil.makeDeploy(
    new DeployUtil.DeployParams(
      pairKeyView.publicKey,
      //"casper-net-1",
      "casper-test",
      constants.DEPLOY_GAS_PRICE,
      constants.DEPLOY_TTL_MS
    ),
    DeployUtil.ExecutableDeployItem.newModuleBytes(
      utils.getBinary(newPath),
      RuntimeArgs.fromMap({
        the_contract_owner: csp_dev,
        collection_name: collection_name,
        collection_symbol: collection_symbol,
        total_token_supply: total_token_supply,
        //csp_mint_fee,
        // csp_dev: csp_dev,
        // csp_minter: csp_minter,
        ownership_mode: ownership_mode,
        nft_kind: nft_kind,
        minting_mode: minting_mode,
        holder_mode,
        json_schema,
        allow_minting,
        nft_metadata_kind,
        identifier_mode,
        metadata_mutability,
        owner_reverse_lookup_mode,
        events_mode: events_mode,
        // minter: csp_minter,
        // "minting_start_time": startMint,
        // "minting_end_time": endMint,
        // "minting_price": feeCSPR, //10 CPSR
        // "cspr_receiver": fee_receiver,




        // exp_contract: EXPContract,
        // exp_package_hash: EXPContractHash,
        // fee_change_name: new CLU256("1000000000"), //1000000000
        // fee_change_stamina: new CLU256("1000000000"),
        // fee_change_charisma: new CLU256("1000000000"),
        // fee_change_intelligence: new CLU256("1000000000"),
      })
    ),
    DeployUtil.standardPayment(300000000000) // 170 CSPR IS ENOUGH    165000000000
  );
  deploy = client.signDeploy(deploy, pairKeyView);

  let deployHash = await client.putDeploy(deploy);
  console.log(`deploy hash = ${deployHash}`);

  let result = await Utils.getDeploy(constants.DEPLOY_NODE_ADDRESS, deployHash);
  console.log("result: ", result)

};

main();