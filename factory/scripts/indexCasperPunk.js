const {
  utils,
  helpers,
  CasperContractClient,
} = require("casper-js-client-helper");

const CEP78 = require("./CSP-cep78");
const {
  CLValueBuilder,
  CLPublicKey,
  CLKey,
  CLString,
  CasperClient,
  CLByteArray,
  RuntimeArgs,
  CLAccountHash,
  DeployUtil,
  Keys,
  CLTypeBuilder,
} = require("casper-js-sdk");
const { DEFAULT_TTL } = require("casper-js-client-helper/dist/constants");

const { setClient, contractSimpleGetter, createRecipientAddress } = helpers;

const sleep = (ms) => {
  return new Promise((resolve) => setTimeout(resolve, ms));
};

const getDeploy = async (NODE_URL, deployHash) => {
  const client = new CasperClient(NODE_URL);
  let i = 300;
  while (i != 0) {
    const [deploy, raw] = await client.getDeploy(deployHash);
    if (raw.execution_results.length !== 0) {
      // @ts-ignore
      if (raw.execution_results[0].result.Success) {
        return deploy;
      } else {
        // @ts-ignore
        throw Error(
          "Contract execution: " +
          // @ts-ignore
          raw.execution_results[0].result.Failure.error_message
        );
      }
    } else {
      i--;
      await sleep(1000);
      continue;
    }
  }
  throw Error("Timeout after " + i + "s. Something's wrong");
};

const genRanHex = (size = 64) =>
  [...Array(size)]
    .map(() => Math.floor(Math.random() * 16).toString(16))
    .join("");
const CSPFactory = class {
  constructor(contractHash, nodeAddress, chainName) {
    this.contractHash = contractHash.startsWith("hash-")
      ? contractHash.slice(5)
      : contractHash;
    this.nodeAddress = nodeAddress;
    this.chainName = chainName;
    this.contractClient = new CasperContractClient(nodeAddress, chainName);
  }

  static async createInstance(contractHash, nodeAddress, chainName) {
    let factory = new CSPFactory(contractHash, nodeAddress, chainName);
    await factory.init();
    console.log("NameKey: ", factory.namedKeys)
    return factory;
  }

  async init() {
    console.log("intializing", this.nodeAddress, this.contractHash);
    const { contractPackageHash, namedKeys } = await setClient(
      this.nodeAddress,
      this.contractHash,
      ["request_ids"]
    );
    console.log("done");
    this.contractPackageHash = contractPackageHash;
    this.contractClient.chainName = this.chainName;
    this.contractClient.contractHash = this.contractHash;
    this.contractClient.contractPackageHash = this.contractPackageHash;
    this.contractClient.nodeAddress = this.nodeAddress;
    /* @ts-ignore */
    this.namedKeys = namedKeys;
  }

  async contractOwner() {
    return await contractSimpleGetter(this.nodeAddress, this.contractHash, [
      "contract_owner",
    ]);
  }

  async transferOwner({
    keys,
    newOwner,
    paymentAmount,
    ttl,
  }) {

    if (!paymentAmount) {
      paymentAmount = paymentAmount ? paymentAmount : "1000000000";
      ttl = ttl ? ttl : DEFAULT_TTL;
    }
    let runtimeArgs = {};
    runtimeArgs = RuntimeArgs.fromMap({
      contract_owner: createRecipientAddress(CLPublicKey.fromHex(newOwner)),
    })
    console.log("sending");
    console.log(paymentAmount)
    console.log(ttl)
    let trial = 5;
    while (true) {
      try {
        let hash = await this.contractClient.contractCall({
          entryPoint: "transfer_owner",
          keys: keys,
          paymentAmount,
          runtimeArgs,
          cb: (deployHash) => {
            console.log("deployHash", deployHash);
          },
          ttl,
        });

        return hash;
      } catch (e) {
        trial--
        if (trial == 0) {
          throw e;
        }
        console.log('waiting 3 seconds')
        await sleep(3000)
      }
    }
  }

  async changeFeeReceiver({
    keys,
    newReceiver,
    paymentAmount,
    ttl,
  }) {

    if (!paymentAmount) {
      paymentAmount = paymentAmount ? paymentAmount : "1000000000";
      ttl = ttl ? ttl : DEFAULT_TTL;
    }
    let runtimeArgs = {};
    runtimeArgs = RuntimeArgs.fromMap({
      fee_receiver: createRecipientAddress(CLPublicKey.fromHex(newReceiver)),
    })
    console.log("sending");
    console.log(paymentAmount)
    console.log(ttl)
    let trial = 5;
    while (true) {
      try {
        let hash = await this.contractClient.contractCall({
          entryPoint: "change_fee_receiver",
          keys: keys,
          paymentAmount,
          runtimeArgs,
          cb: (deployHash) => {
            console.log("deployHash", deployHash);
          },
          ttl,
        });

        return hash;
      } catch (e) {
        trial--
        if (trial == 0) {
          throw e;
        }
        console.log('waiting 3 seconds')
        await sleep(3000)
      }
    }
  }

  async changeMintFee({
    keys,
    newFee,
    paymentAmount,
    ttl,
  }) {

    if (!paymentAmount) {
      paymentAmount = paymentAmount ? paymentAmount : "1000000000";
      ttl = ttl ? ttl : DEFAULT_TTL;
    }
    let runtimeArgs = {};
    runtimeArgs = RuntimeArgs.fromMap({
      wcspr_mint_fee: CLValueBuilder.u256(newFee),
    })
    console.log("sending");
    console.log(paymentAmount)
    console.log(ttl)
    let trial = 5;
    while (true) {
      try {
        let hash = await this.contractClient.contractCall({
          entryPoint: "change_mint_fee",
          keys: keys,
          paymentAmount,
          runtimeArgs,
          cb: (deployHash) => {
            console.log("deployHash", deployHash);
          },
          ttl,
        });

        return hash;
      } catch (e) {
        trial--
        if (trial == 0) {
          throw e;
        }
        console.log('waiting 3 seconds')
        await sleep(3000)
      }
    }
  }


  async changeWcsprContract({
    keys,
    newWcsprContract,
    paymentAmount,
    ttl,
  }) {

    if (!paymentAmount) {
      paymentAmount = paymentAmount ? paymentAmount : "1000000000";
      ttl = ttl ? ttl : DEFAULT_TTL;
    }
    let runtimeArgs = {};

    let newWcsprByte = new CLByteArray(
      Uint8Array.from(Buffer.from(newWcsprContract, "hex"))
    );

    runtimeArgs = RuntimeArgs.fromMap({
      wcspr_contract: createRecipientAddress(newWcsprByte),
    })
    console.log("sending");
    console.log(paymentAmount)
    console.log(ttl)
    let trial = 5;
    while (true) {
      try {
        let hash = await this.contractClient.contractCall({
          entryPoint: "change_wcspr_contract",
          keys: keys,
          paymentAmount,
          runtimeArgs,
          cb: (deployHash) => {
            console.log("deployHash", deployHash);
          },
          ttl,
        });

        return hash;
      } catch (e) {
        trial--
        if (trial == 0) {
          throw e;
        }
        console.log('waiting 3 seconds')
        await sleep(3000)
      }
    }
  }

  async setAddressesWhitelist({
    keys,
    addressesWhitelistArray, // account-hash-... array
    paymentAmount,
    ttl,
  }) {

    if (!paymentAmount) {
      paymentAmount = paymentAmount ? paymentAmount : "15000000000";
      ttl = ttl ? ttl : DEFAULT_TTL;
    }
    let addressesWhitelistArr = addressesWhitelistArray.map((e) => CLValueBuilder.string(e));
    let arr = []
    for (let i = 0; i < addressesWhitelistArr.length; i++) {
      addressesWhitelistArray[i] = CLPublicKey.fromHex(addressesWhitelistArray[i])
      addressesWhitelistArray[i] = createRecipientAddress(addressesWhitelistArray[i])

      arr.push(addressesWhitelistArray[i])

    }
    console.log("ARR length: ", arr.length)
    console.log("arr: ", arr)
    let runtimeArgs = {};

    runtimeArgs = RuntimeArgs.fromMap({
      "new_addresses_whitelist": CLValueBuilder.list(arr),
      "is_whitelist": CLValueBuilder.bool(true)
    })
    console.log("sending");
    console.log(paymentAmount)
    console.log(ttl)
    let trial = 5;
    while (true) {
      try {
        let hash = await this.contractClient.contractCall({
          entryPoint: "set_addresses_whitelist",
          keys: keys,
          paymentAmount,
          runtimeArgs,
          cb: (deployHash) => {
            console.log("deployHash", deployHash);
          },
          ttl,
        });

        return hash;
      } catch (e) {
        trial--
        if (trial == 0) {
          throw e;
        }
        console.log('waiting 3 seconds')
        await sleep(3000)
      }
    }
  }

  async updateAddressesWhitelist({
    keys,
    addressesWhitelistArray, // account-hash-... array
    numberOfTickets,
    paymentAmount,
    ttl,
  }) {

    if (!paymentAmount) {
      paymentAmount = paymentAmount ? paymentAmount : "7000000000";
      ttl = ttl ? ttl : DEFAULT_TTL;
    }
    let addressesWhitelistArr = addressesWhitelistArray.map((e) => CLValueBuilder.string(e));
    let arr = []
    for (let i = 0; i < addressesWhitelistArr.length; i++) {
      addressesWhitelistArray[i] = CLPublicKey.fromHex(addressesWhitelistArray[i])
      addressesWhitelistArray[i] = createRecipientAddress(addressesWhitelistArray[i])

      arr.push(addressesWhitelistArray[i])

    }
    console.log("ARR length: ", arr.length)
    console.log("arr: ", arr)
    let runtimeArgs = {};

    runtimeArgs = RuntimeArgs.fromMap({
      "new_addresses_whitelist": CLValueBuilder.list(arr),
      "number_of_tickets": CLValueBuilder.u8(numberOfTickets)
    })
    console.log("sending");
    console.log(paymentAmount)
    console.log(ttl)
    let trial = 5;
    while (true) {
      try {
        let hash = await this.contractClient.contractCall({
          entryPoint: "update_addresses_whitelist",
          keys: keys,
          paymentAmount,
          runtimeArgs,
          cb: (deployHash) => {
            console.log("deployHash", deployHash);
          },
          ttl,
        });

        return hash;
      } catch (e) {
        trial--
        if (trial == 0) {
          throw e;
        }
        console.log('waiting 3 seconds')
        await sleep(3000)
      }
    }
  }


  async mint({
    keys,
    nftContractHash, // contract CEP78
    paymentAmount,
    metadata,
    ttl,
  }) {
    if (!paymentAmount) {
      paymentAmount = paymentAmount ? paymentAmount : "10000000000";
      ttl = ttl ? ttl : "300000";
    }

    // CEP78 NFT CONTRACT
    console.log("nftContractHash: ", nftContractHash)
    nftContractHash = nftContractHash.startsWith("hash-")
      ? nftContractHash.slice(5)
      : nftContractHash;
    console.log("nftContractHash", nftContractHash);
    nftContractHash = new CLByteArray(
      Uint8Array.from(Buffer.from(nftContractHash, "hex"))
    );
    let nftCep78Hash = createRecipientAddress(nftContractHash)
    console.log("nftCep78Hash: ", nftCep78Hash)


    // NFT METADATA
    const token_meta_data = new CLString(JSON.stringify(metadata))

    console.log("token_meta_data", token_meta_data)

    let runtimeArgs = RuntimeArgs.fromMap({
      "nft_contract_package": nftCep78Hash,
      "token_meta_data": token_meta_data,
    })

    console.log("sending");
    let trial = 5;
    while (true) {
      try {
        let hash = await this.contractClient.contractCall({
          entryPoint: "mint",
          keys: keys,
          paymentAmount,
          runtimeArgs,
          cb: (deployHash) => {
            console.log("deployHash", deployHash);
          },
          ttl,
        });

        return hash;
      } catch (e) {
        trial--
        if (trial == 0) {
          throw e;
        }
        console.log('waiting 3 seconds')
        await sleep(3000)
      }
    }
  }
};
const CSPMarketPlace = class {
  constructor(contractHash, nodeAddress, chainName, namedKeysList = []) {
    this.contractHash = contractHash.startsWith("hash-")
      ? contractHash.slice(5)
      : contractHash;
    this.nodeAddress = nodeAddress;
    this.chainName = chainName;
    this.contractClient = new CasperContractClient(nodeAddress, chainName);
    this.namedKeysList = [
      "contract_hash",
      "contract_owner",
      "market_fee",
      "market_fee_receiver",
      "token_contract_hash",
      "token_contract_map",
      "token_market",
      "token_market_list",
    ];
    this.namedKeysList.push(...namedKeysList)

  }

  static async createInstance(contractHash, nodeAddress, chainName, namedKeysList = []) {
    let market = new CSPMarketPlace(contractHash, nodeAddress, chainName, namedKeysList);
    await market.init();
    console.log("NameKey: ", market.namedKeys)
    return market;
  }

  async init() {
    console.log("intializing", this.nodeAddress, this.contractHash);
    const { contractPackageHash, namedKeys } = await setClient(
      this.nodeAddress,
      this.contractHash,
      this.namedKeysList
    );
    console.log("done");
    this.contractPackageHash = contractPackageHash;
    this.contractClient.chainName = this.chainName;
    this.contractClient.contractHash = this.contractHash;
    this.contractClient.contractPackageHash = this.contractPackageHash;
    this.contractClient.nodeAddress = this.nodeAddress;
    /* @ts-ignore */
    this.namedKeys = namedKeys;
  }

  async contractOwner() {
    return await contractSimpleGetter(this.nodeAddress, this.contractHash, [
      "contract_owner"
    ]);
  }
  async tokenMarketList() {
    return await contractSimpleGetter(this.nodeAddress, this.contractHash, [
      "token_market_list"
    ]);
  }

  async tokenSupportedList() {
    return await contractSimpleGetter(this.nodeAddress, this.contractHash, [
      "token_contract_hash"
    ]);
  }

  async requestIndex() {
    return await contractSimpleGetter(this.nodeAddress, this.contractHash, [
      "request_index",
    ]);
  }

  async getIndexFromRequestId(requestId) {
    try {
      const itemKey = requestId.toString();
      const result = await utils.contractDictionaryGetter(
        this.nodeAddress,
        itemKey,
        this.namedKeys.requestIds
      );
      return result;
    } catch (e) {
      throw e;
    }
  }

  async offer({
    keys,
    nftContractHash, // contract CSP
    minimumOffer, // account-hash-... array
    identifierMode,
    tokenId,
    paymentAmount,
    ttl,
  }) {

    if (!paymentAmount) {
      paymentAmount = paymentAmount ? paymentAmount : "50000000000";
      ttl = ttl ? ttl : DEFAULT_TTL;
    }

    if (identifierMode == undefined) {
      let nftContract = new CEP78(
        nftContractHash,
        this.nodeAddress,
        this.chainName
      );
      await nftContract.init();
      identifierMode = await nftContract.identifierMode();
    }
    nftContractHash = new CLByteArray(
      Uint8Array.from(Buffer.from(nftContractHash, "hex"))
    );
    let runtimeArgs = {};
    if (identifierMode == 0) {
      runtimeArgs = RuntimeArgs.fromMap({
        token_id: CLValueBuilder.u64(tokenId),
        nft_contract_hash: createRecipientAddress(nftContractHash),
        minimum_offer: CLValueBuilder.u256(minimumOffer),
        identifier_mode: CLValueBuilder.u8(identifierMode),
      })
    } else {
      runtimeArgs = RuntimeArgs.fromMap({
        token_hashes: CLValueBuilder.string(tokenId),
        nft_contract_hash: createRecipientAddress(nftContractHash),
        minimum_offer: CLValueBuilder.u256(minimumOffer),
        identifier_mode: CLValueBuilder.u8(identifierMode)

      });
    }
    console.log("sending");
    console.log(paymentAmount)
    console.log(ttl)
    let trial = 5;
    while (true) {
      try {
        let hash = await this.contractClient.contractCall({
          entryPoint: "offer",
          keys: keys,
          paymentAmount,
          runtimeArgs,
          cb: (deployHash) => {
            console.log("deployHash", deployHash);
          },
          ttl,
        });

        return hash;
      } catch (e) {
        trial--
        if (trial == 0) {
          throw e;
        }
        console.log('waiting 3 seconds')
        await sleep(3000)
      }
    }
  }

  async revokeOffer({
    keys,
    nftContractHash, // contract CSP
    identifierMode,
    tokenId,
    paymentAmount,
    ttl,
  }) {

    if (!paymentAmount) {
      paymentAmount = paymentAmount ? paymentAmount : "30000000000";
      ttl = ttl ? ttl : DEFAULT_TTL;
    }

    if (identifierMode == undefined) {
      let nftContract = new CEP78(
        nftContractHash,
        this.nodeAddress,
        this.chainName
      );
      await nftContract.init();
      identifierMode = await nftContract.identifierMode();
    }
    nftContractHash = new CLByteArray(
      Uint8Array.from(Buffer.from(nftContractHash, "hex"))
    );
    let runtimeArgs = {};
    if (identifierMode == 0) {
      runtimeArgs = RuntimeArgs.fromMap({
        token_id: CLValueBuilder.u64(tokenId),
        nft_contract_hash: createRecipientAddress(nftContractHash),
        identifier_mode: CLValueBuilder.u8(identifierMode),
      })
    } else {
      runtimeArgs = RuntimeArgs.fromMap({
        token_hashes: CLValueBuilder.string(tokenId),
        nft_contract_hash: createRecipientAddress(nftContractHash),
        identifier_mode: CLValueBuilder.u8(identifierMode)

      });
    }
    console.log("sending");
    console.log(paymentAmount)
    console.log(ttl)
    let trial = 5;
    while (true) {
      try {
        let hash = await this.contractClient.contractCall({
          entryPoint: "revoke_offer",
          keys: keys,
          paymentAmount,
          runtimeArgs,
          cb: (deployHash) => {
            console.log("deployHash", deployHash);
          },
          ttl,
        });

        return hash;
      } catch (e) {
        trial--
        if (trial == 0) {
          throw e;
        }
        console.log('waiting 3 seconds')
        await sleep(3000)
      }
    }
  }


  async changeOffer({
    keys,
    nftContractHash, // contract CSP
    identifierMode,
    tokenId,
    newMinimum,
    paymentAmount,
    ttl,
  }) {

    if (!paymentAmount) {
      paymentAmount = paymentAmount ? paymentAmount : "50000000000";
      ttl = ttl ? ttl : DEFAULT_TTL;
    }

    if (identifierMode == undefined) {
      let nftContract = new CEP78(
        nftContractHash,
        this.nodeAddress,
        this.chainName
      );
      await nftContract.init();
      identifierMode = await nftContract.identifierMode();
    }
    nftContractHash = new CLByteArray(
      Uint8Array.from(Buffer.from(nftContractHash, "hex"))
    );
    let runtimeArgs = {};
    if (identifierMode == 0) {
      runtimeArgs = RuntimeArgs.fromMap({
        token_id: CLValueBuilder.u64(tokenId),
        nft_contract_hash: createRecipientAddress(nftContractHash),
        identifier_mode: CLValueBuilder.u8(identifierMode),
        new_minimum_offer: CLValueBuilder.u256(newMinimum),
      })
    } else {
      runtimeArgs = RuntimeArgs.fromMap({
        token_hashes: CLValueBuilder.string(tokenId),
        nft_contract_hash: createRecipientAddress(nftContractHash),
        identifier_mode: CLValueBuilder.u8(identifierMode),
        new_minimum_offer: CLValueBuilder.u256(newMinimum),

      });
    }
    console.log("sending");
    console.log(paymentAmount)
    console.log(ttl)
    let trial = 5;
    while (true) {
      try {
        let hash = await this.contractClient.contractCall({
          entryPoint: "change_offer",
          keys: keys,
          paymentAmount,
          runtimeArgs,
          cb: (deployHash) => {
            console.log("deployHash", deployHash);
          },
          ttl,
        });

        return hash;
      } catch (e) {
        trial--
        if (trial == 0) {
          throw e;
        }
        console.log('waiting 3 seconds')
        await sleep(3000)
      }
    }
  }

  async revokeBid({
    keys,
    nftContractHash, // contract CSP
    identifierMode,
    tokenId,
    paymentAmount,
    ttl,
  }) {

    if (!paymentAmount) {
      paymentAmount = paymentAmount ? paymentAmount : "40000000000";
      ttl = ttl ? ttl : DEFAULT_TTL;
    }

    if (identifierMode == undefined) {
      let nftContract = new CEP78(
        nftContractHash,
        this.nodeAddress,
        this.chainName
      );
      await nftContract.init();
      identifierMode = await nftContract.identifierMode();
    }
    nftContractHash = new CLByteArray(
      Uint8Array.from(Buffer.from(nftContractHash, "hex"))
    );
    let runtimeArgs = {};
    if (identifierMode == 0) {
      runtimeArgs = RuntimeArgs.fromMap({
        token_id: CLValueBuilder.u64(tokenId),
        nft_contract_hash: createRecipientAddress(nftContractHash),
        identifier_mode: CLValueBuilder.u8(identifierMode),
      })
    } else {
      runtimeArgs = RuntimeArgs.fromMap({
        token_hashes: CLValueBuilder.string(tokenId),
        nft_contract_hash: createRecipientAddress(nftContractHash),
        identifier_mode: CLValueBuilder.u8(identifierMode)

      });
    }
    console.log("sending");
    console.log(paymentAmount)
    console.log(ttl)
    let trial = 5;
    while (true) {
      try {
        let hash = await this.contractClient.contractCall({
          entryPoint: "revoke_bid",
          keys: keys,
          paymentAmount,
          runtimeArgs,
          cb: (deployHash) => {
            console.log("deployHash", deployHash);
          },
          ttl,
        });

        return hash;
      } catch (e) {
        trial--
        if (trial == 0) {
          throw e;
        }
        console.log('waiting 3 seconds')
        await sleep(3000)
      }
    }
  }

  async bid({
    keys,
    nftContractHash, // contract CSP
    biddingOffer,
    identifierMode,
    tokenId,
    paymentAmount,
    ttl,
  }) {

    if (!paymentAmount) {
      paymentAmount = paymentAmount ? paymentAmount : "40000000000";
      ttl = ttl ? ttl : DEFAULT_TTL;
    }

    if (identifierMode == undefined) {
      let nftContract = new CEP78(
        nftContractHash,
        this.nodeAddress,
        this.chainName
      );
      await nftContract.init();
      identifierMode = await nftContract.identifierMode();
    }
    nftContractHash = new CLByteArray(
      Uint8Array.from(Buffer.from(nftContractHash, "hex"))
    );
    let runtimeArgs = {};
    if (identifierMode == 0) {
      runtimeArgs = RuntimeArgs.fromMap({
        token_id: CLValueBuilder.u64(tokenId),
        nft_contract_hash: createRecipientAddress(nftContractHash),
        bidding_offer: CLValueBuilder.u256(biddingOffer),
        identifier_mode: CLValueBuilder.u8(identifierMode),
      })
    } else {
      runtimeArgs = RuntimeArgs.fromMap({
        token_hashes: CLValueBuilder.string(tokenId),
        nft_contract_hash: createRecipientAddress(nftContractHash),
        bidding_offer: CLValueBuilder.u256(biddingOffer),
        identifier_mode: CLValueBuilder.u8(identifierMode)

      });
    }
    console.log("runtimeArgs : ", runtimeArgs)
    console.log("sending");
    console.log(paymentAmount)
    console.log(ttl)
    let trial = 5;
    while (true) {
      try {
        let hash = await this.contractClient.contractCall({
          entryPoint: "bid",
          keys: keys,
          paymentAmount,
          runtimeArgs,
          cb: (deployHash) => {
            console.log("deployHash", deployHash);
          },
          ttl,
        });

        return hash;
      } catch (e) {
        trial--
        if (trial == 0) {
          throw e;
        }
        console.log('waiting 3 seconds')
        await sleep(3000)
      }
    }
  }


  async increaseBid({
    keys,
    nftContractHash, // contract CSP
    newOffer,
    identifierMode,
    tokenId,
    paymentAmount,
    ttl,
  }) {

    if (!paymentAmount) {
      paymentAmount = paymentAmount ? paymentAmount : "40000000000";
      ttl = ttl ? ttl : DEFAULT_TTL;
    }

    if (identifierMode == undefined) {
      let nftContract = new CEP78(
        nftContractHash,
        this.nodeAddress,
        this.chainName
      );
      await nftContract.init();
      identifierMode = await nftContract.identifierMode();
    }
    nftContractHash = new CLByteArray(
      Uint8Array.from(Buffer.from(nftContractHash, "hex"))
    );
    let runtimeArgs = {};
    if (identifierMode == 0) {
      runtimeArgs = RuntimeArgs.fromMap({
        token_id: CLValueBuilder.u64(tokenId),
        nft_contract_hash: createRecipientAddress(nftContractHash),
        new_offer: CLValueBuilder.u256(newOffer),
        identifier_mode: CLValueBuilder.u8(identifierMode),
      })
    } else {
      runtimeArgs = RuntimeArgs.fromMap({
        token_hashes: CLValueBuilder.string(tokenId),
        nft_contract_hash: createRecipientAddress(nftContractHash),
        new_offer: CLValueBuilder.u256(newOffer),
        identifier_mode: CLValueBuilder.u8(identifierMode)

      });
    }
    console.log("runtimeArgs : ", runtimeArgs)
    console.log("sending");
    console.log(paymentAmount)
    console.log(ttl)
    let trial = 5;
    while (true) {
      try {
        let hash = await this.contractClient.contractCall({
          entryPoint: "increase_bid",
          keys: keys,
          paymentAmount,
          runtimeArgs,
          cb: (deployHash) => {
            console.log("deployHash", deployHash);
          },
          ttl,
        });

        return hash;
      } catch (e) {
        trial--
        if (trial == 0) {
          throw e;
        }
        console.log('waiting 3 seconds')
        await sleep(3000)
      }
    }
  }


  async setSupportToken({
    keys,
    nftContractHash, // contract CSP
    nftEnabled,
    paymentAmount,
    ttl,
  }) {

    if (!paymentAmount) {
      paymentAmount = paymentAmount ? paymentAmount : "5000000000";
      ttl = ttl ? ttl : DEFAULT_TTL;
    }

    nftContractHash = new CLByteArray(
      Uint8Array.from(Buffer.from(nftContractHash, "hex"))
    );
    let runtimeArgs = {};
    runtimeArgs = RuntimeArgs.fromMap({
      nft_enabled: CLValueBuilder.bool(nftEnabled),
      nft_contract_hash: createRecipientAddress(nftContractHash),
    })
    console.log("sending");
    console.log(paymentAmount)
    console.log(ttl)
    let trial = 5;
    while (true) {
      try {
        let hash = await this.contractClient.contractCall({
          entryPoint: "set_support_token",
          keys: keys,
          paymentAmount,
          runtimeArgs,
          cb: (deployHash) => {
            console.log("deployHash", deployHash);
          },
          ttl,
        });

        return hash;
      } catch (e) {
        trial--
        if (trial == 0) {
          throw e;
        }
        console.log('waiting 3 seconds')
        await sleep(3000)
      }
    }
  }
  async transferOwner({
    keys,
    newOwner, // contract CSP
    paymentAmount,
    ttl,
  }) {

    if (!paymentAmount) {
      paymentAmount = paymentAmount ? paymentAmount : "10000000000";
      ttl = ttl ? ttl : DEFAULT_TTL;
    }
    let runtimeArgs = {};
    runtimeArgs = RuntimeArgs.fromMap({
      contract_owner: createRecipientAddress(CLPublicKey.fromHex(newOwner)),
    })
    console.log("sending");
    console.log(paymentAmount)
    console.log(ttl)
    let trial = 5;
    while (true) {
      try {
        let hash = await this.contractClient.contractCall({
          entryPoint: "transfer_owner",
          keys: keys,
          paymentAmount,
          runtimeArgs,
          cb: (deployHash) => {
            console.log("deployHash", deployHash);
          },
          ttl,
        });

        return hash;
      } catch (e) {
        trial--
        if (trial == 0) {
          throw e;
        }
        console.log('waiting 3 seconds')
        await sleep(3000)
      }
    }
  }
  async changeWcsprContract({
    keys,
    newWcsprContract, // contract CSP
    paymentAmount,
    ttl,
  }) {

    if (!paymentAmount) {
      paymentAmount = paymentAmount ? paymentAmount : "10000000000";
      ttl = ttl ? ttl : DEFAULT_TTL;
    }
    let newWcsprByte = new CLByteArray(
      Uint8Array.from(Buffer.from(newWcsprContract, "hex"))
    );

    let runtimeArgs = {};
    runtimeArgs = RuntimeArgs.fromMap({
      wcspr_contract: createRecipientAddress(newWcsprByte),
    })
    console.log("sending");
    console.log(paymentAmount)
    console.log(ttl)
    let trial = 5;
    while (true) {
      try {
        let hash = await this.contractClient.contractCall({
          entryPoint: "change_wcspr_contract",
          keys: keys,
          paymentAmount,
          runtimeArgs,
          cb: (deployHash) => {
            console.log("deployHash", deployHash);
          },
          ttl,
        });

        return hash;
      } catch (e) {
        trial--
        if (trial == 0) {
          throw e;
        }
        console.log('waiting 3 seconds')
        await sleep(3000)
      }
    }
  }

  async changeIsRoyalty({
    keys,
    isRoyalty, // 
    paymentAmount,
    ttl,
  }) {

    if (!paymentAmount) {
      paymentAmount = paymentAmount ? paymentAmount : "10000000000";
      ttl = ttl ? ttl : DEFAULT_TTL;
    }
    let runtimeArgs = {};
    runtimeArgs = RuntimeArgs.fromMap({
      is_royalty: CLValueBuilder.bool(isRoyalty),
    })
    console.log("sending");
    console.log(paymentAmount)
    console.log(ttl)
    let trial = 5;
    while (true) {
      try {
        let hash = await this.contractClient.contractCall({
          entryPoint: "change_is_royalty",
          keys: keys,
          paymentAmount,
          runtimeArgs,
          cb: (deployHash) => {
            console.log("deployHash", deployHash);
          },
          ttl,
        });

        return hash;
      } catch (e) {
        trial--
        if (trial == 0) {
          throw e;
        }
        console.log('waiting 3 seconds')
        await sleep(3000)
      }
    }
  }

  async changeRoyaltyFee({
    keys,
    royaltyFee, // contract CSP
    paymentAmount,
    ttl,
  }) {

    if (!paymentAmount) {
      paymentAmount = paymentAmount ? paymentAmount : "10000000000";
      ttl = ttl ? ttl : DEFAULT_TTL;
    }
    let runtimeArgs = {};
    runtimeArgs = RuntimeArgs.fromMap({
      royalty_fee: CLValueBuilder.u256(royaltyFee),
    })
    console.log("sending");
    console.log(paymentAmount)
    console.log(ttl)
    let trial = 5;
    while (true) {
      try {
        let hash = await this.contractClient.contractCall({
          entryPoint: "change_royalty_fee",
          keys: keys,
          paymentAmount,
          runtimeArgs,
          cb: (deployHash) => {
            console.log("deployHash", deployHash);
          },
          ttl,
        });

        return hash;
      } catch (e) {
        trial--
        if (trial == 0) {
          throw e;
        }
        console.log('waiting 3 seconds')
        await sleep(3000)
      }
    }
  }

  async changeMaketFee({
    keys,
    marketFee, // contract CSP
    paymentAmount,
    ttl,
  }) {

    if (!paymentAmount) {
      paymentAmount = paymentAmount ? paymentAmount : "10000000000";
      ttl = ttl ? ttl : DEFAULT_TTL;
    }
    let runtimeArgs = {};
    runtimeArgs = RuntimeArgs.fromMap({
      market_fee: CLValueBuilder.u256(marketFee),
    })
    console.log("sending");
    console.log(paymentAmount)
    console.log(ttl)
    let trial = 5;
    while (true) {
      try {
        let hash = await this.contractClient.contractCall({
          entryPoint: "change_fee",
          keys: keys,
          paymentAmount,
          runtimeArgs,
          cb: (deployHash) => {
            console.log("deployHash", deployHash);
          },
          ttl,
        });

        return hash;
      } catch (e) {
        trial--
        if (trial == 0) {
          throw e;
        }
        console.log('waiting 3 seconds')
        await sleep(3000)
      }
    }
  }

};

module.exports = { CSPFactory, genRanHex, CEP78, CSPMarketPlace };
