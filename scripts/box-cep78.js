const {
    utils,
    helpers,
    CasperContractClient,
} = require("casper-js-client-helper");
const { DEFAULT_TTL } = require("casper-js-client-helper/dist/constants");

const { CLValueBuilder, RuntimeArgs, CLAccountHash, CLString, CLPublicKey, CLKey, CLByteArray, CLValueParsers } = require("casper-js-sdk");

const { setClient, contractSimpleGetter, createRecipientAddress } = helpers;
let blake = require("blakejs")

const getOperatorDictionaryKey = (caller, operator) => {
    let callerKey = createRecipientAddress(CLPublicKey.fromHex(caller))
    const contracthashbytearray = new CLByteArray(Uint8Array.from(Buffer.from(operator, 'hex')));
    const operatorKey = new CLKey(contracthashbytearray);
    let callerKeyBytes = CLValueParsers.toBytes(callerKey).val
    let operatorKeyBytes = CLValueParsers.toBytes(operatorKey).val
    let mix = Array.from(callerKeyBytes).concat(Array.from(operatorKeyBytes))
    let result = blake.blake2b(Buffer.from(mix), null, 32)
    console.log('h1', Buffer.from(result).toString('hex'))
    return result
}


const CEP78 = class {
    constructor(contractHash, nodeAddress, chainName, namedKeysList = []) {
        this.contractHash = contractHash.startsWith("hash-")
            ? contractHash.slice(5)
            : contractHash;
        this.nodeAddress = nodeAddress;
        this.chainName = chainName;
        this.contractClient = new CasperContractClient(nodeAddress, chainName);
        this.namedKeysList = [
            "balances",
            "burnt_tokens",
            "metadata_cep78",
            "token_owners",
            "metadata_custom_validated",
            "metadata_nft721",
            "metadata_raw",
            "operators",
            "owned_tokens",
            "token_issuers",
            "page_table",
            "page_0",
            "page_1",
            "page_2",
            "page_3",
            "page_4",
            "page_5",
            "page_6",
            "page_7",
            "page_8",
            "page_9",
            "page_10",
            "user_mint_id_list",
            "hash_by_index",
            "events",
            "index_by_hash",
            "receipt_name",
            "rlo_mflag",
            "reporting_mode",
            // "token_owners",

        ];
        this.namedKeysList.push(...namedKeysList)

    }

    static async createInstance(contractHash, nodeAddress, chainName, namedKeysList = []) {
        let wNFT = new CEP78(contractHash, nodeAddress, chainName, namedKeysList);
        await wNFT.init();
        return wNFT;
    }

    NFTMetadataKind = {
        CEP78: 0,
        NFT721: 1,
        Raw: 2,
        CustomValidated: 3,
    };

    async init() {
        const { contractPackageHash, namedKeys } = await setClient(
            this.nodeAddress,
            this.contractHash,
            this.namedKeysList
        );
        this.contractPackageHash = contractPackageHash;
        this.contractClient.chainName = this.chainName
        this.contractClient.contractHash = this.contractHash
        this.contractClient.contractPackageHash = this.contractPackageHash
        this.contractClient.nodeAddress = this.nodeAddress
        this.namedKeys = namedKeys;
    }

    async identifierMode() {
        let mode = await contractSimpleGetter(this.nodeAddress, this.contractHash, [
            "identifier_mode",
        ]);
        return mode.toNumber()
    }

    async collectionName() {
        return await this.readContractField("collection_name");
    }

    async allowMinting() {
        return await this.readContractField("allow_minting");
    }

    async collectionSymbol() {
        return await this.readContractField("collection_symbol");
    }

    async contractWhitelist() {
        return await this.readContractField("contract_whitelist");
    }

    async holderMode() {
        return await this.readContractField("holder_mode");
    }

    async installer() {
        return await this.readContractField("installer");
    }

    async jsonSchema() {
        return await this.readContractField("json_schema");
    }

    async metadataMutability() {
        return await this.readContractField("metadata_mutability");
    }

    async mintingMode() {
        return await this.readContractField("minting_mode");
    }

    async nftKind() {
        return await this.readContractField("nft_kind");
    }

    async nftMetadataKind() {
        return await this.readContractField("nft_metadata_kind");
    }

    async numberOfMintedTokens() {
        return await this.readContractField("number_of_minted_tokens");
    }

    async ownershipMode() {
        return await this.readContractField("ownership_mode");
    }

    async receiptName() {
        return await this.readContractField("receipt_name");
    }

    async totalTokenSupply() {
        return await this.readContractField("total_token_supply");
    }

    async whitelistMode() {
        return await this.readContractField("whitelist_mode");
    }

    async readContractField(field) {
        return await contractSimpleGetter(this.nodeAddress, this.contractHash, [
            field,
        ]);
    }

    async getOperator(tokenId) {
        try {
            const itemKey = tokenId.toString();
            const result = await utils.contractDictionaryGetter(
                this.nodeAddress,
                itemKey,
                this.namedKeys.operator
            );
            return Buffer.from(result.val.data.data).toString("hex");
        } catch (e) {
            throw e;
        }
    }

    async getOwnerOf(tokenId) {
        try {
            const itemKey = tokenId.toString();
            console.log(itemKey)
            console.log(this.namedKeys.tokenOwners)
            const result = await utils.contractDictionaryGetter(
                this.nodeAddress,
                itemKey,
                this.namedKeys.tokenOwners
            );
            return Buffer.from(result.data).toString("hex");
        } catch (e) {
            throw e;
        }
    }

    async burntTokens(tokenId) {
        try {
            const itemKey = tokenId.toString();
            const result = await utils.contractDictionaryGetter(
                this.nodeAddress,
                itemKey,
                this.namedKeys.burntTokens
            );
            return result ? true : false;
        } catch (e) { }
        return false;
    }

    async getTokenMetadata(tokenId) {
        try {
            const itemKey = tokenId.toString();
            let nftMetadataKind = await this.nftMetadataKind();
            nftMetadataKind = parseInt(nftMetadataKind.toString());
            let result = null;
            if (nftMetadataKind == this.NFTMetadataKind.CEP78) {
                result = await utils.contractDictionaryGetter(
                    this.nodeAddress,
                    itemKey,
                    this.namedKeys.metadataCep78
                );
            } else if (nftMetadataKind == this.NFTMetadataKind.CustomValidated) {
                result = await utils.contractDictionaryGetter(
                    this.nodeAddress,
                    itemKey,
                    this.namedKeys.metadataCustomValidated
                );
            } else if (nftMetadataKind == this.NFTMetadataKind.NFT721) {
                result = await utils.contractDictionaryGetter(
                    this.nodeAddress,
                    itemKey,
                    this.namedKeys.metadataNft721
                );
            } else if (nftMetadataKind == this.NFTMetadataKind.Raw) {
                result = await utils.contractDictionaryGetter(
                    this.nodeAddress,
                    itemKey,
                    this.namedKeys.metadataRaw
                );
            }
            // } else if (nftMetadataKind == this.NFTMetadataKind.CasperPunk) {
            //     console.log(this.namedKeys.metadataCasperpunk)
            //     result = await utils.contractDictionaryGetter(
            //         this.nodeAddress,
            //         itemKey,
            //         this.namedKeys.metadataCasperpunk
            //         //"uref-d97097a04ee5957aacb78859843476b01290fd39bb81cd32d6e5d4a66e2593ee-007"
            //     );
            // }


            return result;
        } catch (e) {
            throw e;
        }
    }

    static getAccountItemKey(account) {
        let itemKey = "";
        if (typeof account == String) {
            itemKey = account.toString();
        } else {
            let key = createRecipientAddress(account);
            itemKey = Buffer.from(key.data.data).toString("hex");
        }
        return itemKey;
    }

    async checkRegisterOwner(ownerAccountHash) {
        try {
            const itemKey = ownerAccountHash.toString();
            const result = await utils.contractDictionaryGetter(
                this.nodeAddress,
                itemKey,
                this.namedKeys.pageTable
            );
            return result;
        } catch (e) {
            throw e;
        }
    }


    async getOwnedTokens(account) {
        try {
            let itemKey = CEP78.getAccountItemKey(account);
            const result = await utils.contractDictionaryGetter(
                this.nodeAddress,
                itemKey,
                this.namedKeys.ownedTokens
            );
            return result.map((e) => e.data);
        } catch (e) {
            throw e;
        }
    }

    async userMintIdList(account) {
        try {
            let itemKey = CEP78.getAccountItemKey(account);
            console.log("itemKey ", itemKey)
            console.log("here", this.namedKeys.userMintIdList)
            const result = await utils.contractDictionaryGetter(
                this.nodeAddress,
                itemKey,
                this.namedKeys.userMintIdList
            );
            console.log("result", result)
            return result;
        } catch (e) {
            throw e;
        }
    }

    async balanceOf(account) {
        try {
            let itemKey = CEP78.getAccountItemKey(account);
            const result = await utils.contractDictionaryGetter(
                this.nodeAddress,
                itemKey,
                this.namedKeys.balances
            );
            return result;
        } catch (e) {
            throw e;
        }
    }

    async getOwnedTokenIds(account) {
        let table = []
        try {
            let itemKey = CEP78.getAccountItemKey(account);
            console.log(this.namedKeys.pageTable)
            const result = await utils.contractDictionaryGetter(
                this.nodeAddress,
                itemKey,
                this.namedKeys.pageTable
            );


            for (var i = 0; i < result.length; i++) {
                if (result[i].data == true) {
                    table.push(i)
                }
            }

            let tokenIds = []

            for (var j = 0; j < table.length; j++) {

                let k = table[j]

                let numberOfPage = "page_" + k
                console.log(numberOfPage.toString())
                const result = await utils.contractDictionaryGetter(
                    this.nodeAddress,
                    itemKey,
                    this.namedKeys[numberOfPage]
                );
                for (var i = 0; i < result.length; i++) {
                    if (result[i].data == true) {
                        tokenIds.push(i)
                    }
                }

            }
            return tokenIds;
            // return table;
        } catch (e) {
            throw e;
        }


    }

    async getOwnedTokenIdsHash(account) {
        let table = []
        try {
            let itemKey = CEP78.getAccountItemKey(account);
            console.log(this.namedKeys.pageTable)
            const result = await utils.contractDictionaryGetter(
                this.nodeAddress,
                itemKey,
                this.namedKeys.pageTable
            );


            for (var i = 0; i < result.length; i++) {
                if (result[i].data == true) {
                    table.push(i)
                }
            }

            let tokenIds = []

            for (var j = 0; j < table.length; j++) {

                let k = table[j]

                let numberOfPage = "page_" + k
                console.log(numberOfPage.toString())
                const result = await utils.contractDictionaryGetter(
                    this.nodeAddress,
                    itemKey,
                    this.namedKeys[numberOfPage]
                );
                for (var i = 0; i < result.length; i++) {
                    if (result[i].data == true) {
                        tokenIds.push(i)
                    }
                }

            }
            let final = []
            console.log("tokenIds ", tokenIds)
            for (var m = 0; m < tokenIds.length; m++) {
                let string = tokenIds[m].toString()

                console.log(tokenIds[m])
                console.log(this.namedKeys.hashByIndex)
                const result = await utils.contractDictionaryGetter(
                    this.nodeAddress,
                    string,
                    this.namedKeys.hashByIndex
                );
                final.push(result)
            }
            return final;
            // return table;
        } catch (e) {
            throw e;
        }


    }
    async pageTable(account) {
        try {
            let itemKey = CEP78.getAccountItemKey(account);
            console.log(this.namedKeys.pageTable)
            const result = await utils.contractDictionaryGetter(
                this.nodeAddress,
                itemKey,
                this.namedKeys.pageTable
            );

            let table = []

            for (var i = 0; i < result.length; i++) {
                if (result[i].data == true) {
                    table.push(i)
                }
            }
            return table;
        } catch (e) {
            throw e;
        }
    }
    async pageDetails(i, account) {
        try {
            let itemKey = CEP78.getAccountItemKey(account);
            let numberOfPage = "page_" + i
            console.log(numberOfPage.toString())
            console.log(this.namedKeys.page_0)
            const result = await utils.contractDictionaryGetter(
                this.nodeAddress,
                itemKey,
                this.namedKeys[numberOfPage]
            );
            // console.log(result)


            let tokenIds = []

            for (var i = 0; i < result.length; i++) {
                if (result[i].data == true) {
                    tokenIds.push(i)
                }
            }
            return tokenIds;
        } catch (e) {
            throw e;
        }
    }


    async approve(keys, operator, tokenId, paymentAmount, ttl) {
        let key = createRecipientAddress(operator);
        let identifierMode = await this.identifierMode();
        identifierMode = parseInt(identifierMode.toString());
        let runtimeArgs = {};
        if (identifierMode == 0) {
            runtimeArgs = RuntimeArgs.fromMap({
                token_id: CLValueBuilder.u64(parseInt(tokenId)),
                operator: key,
            });
        } else {
            runtimeArgs = RuntimeArgs.fromMap({
                token_hash: CLValueBuilder.string(tokenId),
                operator: key,
            });
        }

        return await this.contractClient.contractCall({
            entryPoint: "approve",
            keys: keys,
            paymentAmount: paymentAmount ? paymentAmount : "1000000000",
            runtimeArgs,
            cb: (deployHash) => { },
            ttl: ttl ? ttl : DEFAULT_TTL,
        });
    }

    async mint({ keys, tokenOwner, metadataJson, paymentAmount, ttl }) {
        // Owner input is accountHash
        tokenOwner = tokenOwner.startsWith("account-hash-")
            ? tokenOwner.slice(13)
            : tokenOwner;


        let ownerAccountHashByte = Uint8Array.from(
            Buffer.from(tokenOwner, 'hex'),
        )


        const ownerKey = createRecipientAddress(new CLAccountHash(ownerAccountHashByte))


        let token_metadata = new CLString(JSON.stringify(metadataJson))
        let runtimeArgs = {};
        runtimeArgs = RuntimeArgs.fromMap({
            token_owner: ownerKey,
            token_meta_data: token_metadata,
        });

        console.log(runtimeArgs)

        return await this.contractClient.contractCall({
            entryPoint: "mint",
            keys: keys,
            paymentAmount: paymentAmount ? paymentAmount : "10000000000",
            runtimeArgs,
            cb: (deployHash) => { },
            ttl: ttl ? ttl : DEFAULT_TTL,
        });
    }

    async mintBoxx({ keys, tokenOwners, numberOfBoxs, metadataJson, paymentAmount, ttl }) {

        let tokenOwnerArray = CLValueBuilder.list(tokenOwners.map(owner => createRecipientAddress(CLPublicKey.fromHex(owner))))

        let numberOfBoxArray = CLValueBuilder.list(numberOfBoxs.map(number => CLValueBuilder.u8(number)))
        let token_metadata = new CLString(JSON.stringify(metadataJson))
        let runtimeArgs = {};
        runtimeArgs = RuntimeArgs.fromMap({
            token_owners: tokenOwnerArray,
            token_meta_data: token_metadata,
            number_of_boxs: numberOfBoxArray
        });
        return await this.contractClient.contractCall({
            entryPoint: "mint",
            keys: keys,
            paymentAmount: paymentAmount ? paymentAmount : "80000000000",
            runtimeArgs,
            cb: (deployHash) => { },
            ttl: ttl ? ttl : DEFAULT_TTL,
        });
    }

    async setMinter({ keys, minter, paymentAmount, ttl }) {
        let contractHash = new CLByteArray(
            Uint8Array.from(Buffer.from(minter, "hex"))
        );

        const minterX = createRecipientAddress(contractHash)


        let runtimeArgs = {};
        runtimeArgs = RuntimeArgs.fromMap({
            the_contract_minter: minterX,
        });


        return await this.contractClient.contractCall({
            entryPoint: "change_minter",
            keys: keys,
            paymentAmount: paymentAmount ? paymentAmount : "10000000000",
            runtimeArgs,
            cb: (deployHash) => { },
            ttl: ttl ? ttl : DEFAULT_TTL,
        });
    }

    async setEnableTransfer({ keys, paymentAmount, ttl }) {

        let runtimeArgs = {};
        runtimeArgs = RuntimeArgs.fromMap({
            enable_transfer: CLValueBuilder.bool(true),
        });

        return await this.contractClient.contractCall({
            entryPoint: "set_enable_transfer",
            keys: keys,
            paymentAmount: paymentAmount ? paymentAmount : "10000000000",
            runtimeArgs,
            cb: (deployHash) => { },
            ttl: ttl ? ttl : DEFAULT_TTL,
        });
    }
    async setParams({ keys, mintingStart, mintingEnd, mintingPrice, paymentAmount, ttl }) {


        let runtimeArgs = {};
        runtimeArgs = RuntimeArgs.fromMap({
            minting_start_time: CLValueBuilder.u64(mintingStart),
            minting_end_time: CLValueBuilder.u64(mintingEnd),
            minting_price: CLValueBuilder.u256(mintingPrice)
        });

        console.log(runtimeArgs)

        return await this.contractClient.contractCall({
            entryPoint: "update_mint_params",
            keys: keys,
            paymentAmount: paymentAmount ? paymentAmount : "10000000000",
            runtimeArgs,
            cb: (deployHash) => { },
            ttl: ttl ? ttl : DEFAULT_TTL,
        });
    }

    async claim({ keys, paymentAmount, ttl }) {
        // Owner input is accountHash
        // tokenOwner = tokenOwner.startsWith("account-hash-")
        //     ? tokenOwner.slice(13)
        //     : tokenOwner;


        // let ownerAccountHashByte = Uint8Array.from(
        //     Buffer.from(tokenOwner, 'hex'),
        // )


        // const ownerKey = createRecipientAddress(new CLAccountHash(ownerAccountHashByte))


        // let token_metadata = new CLString(JSON.stringify(metadataJson))
        let runtimeArgs = {};
        runtimeArgs = RuntimeArgs.fromMap({
            // token_owner: ownerKey,
            // token_meta_data: token_metadata,
        });

        console.log(runtimeArgs)

        return await this.contractClient.contractCall({
            entryPoint: "claim",
            keys: keys,
            paymentAmount: paymentAmount ? paymentAmount : "30000000000",
            runtimeArgs,
            cb: (deployHash) => { },
            ttl: ttl ? ttl : DEFAULT_TTL,
        });
    }

    async registerOwner({ keys, tokenOwner, paymentAmount, ttl }) {

        const ownerKey = createRecipientAddress(CLPublicKey.fromHex(tokenOwner))
        let runtimeArgs = {};
        runtimeArgs = RuntimeArgs.fromMap({
            token_owner: ownerKey,
            // token_meta_data: token_metadata,
        });

        console.log("before")
        return await this.contractClient.contractCall({
            entryPoint: "register_owner",
            keys: keys,
            paymentAmount: paymentAmount ? paymentAmount : "1000000000",
            runtimeArgs,
            cb: (deployHash) => { },
            ttl: ttl ? ttl : DEFAULT_TTL,
        });
    }
    async registerOwnerForContract({ keys, contractHash, paymentAmount, ttl }) {
        contractHash = new CLByteArray(
            Uint8Array.from(Buffer.from(contractHash, "hex"))
        );

        const ownerKey = createRecipientAddress(contractHash)
        let runtimeArgs = {};
        runtimeArgs = RuntimeArgs.fromMap({
            token_owner: ownerKey,
            // token_meta_data: token_metadata,
        });

        console.log("before")
        return await this.contractClient.contractCall({
            entryPoint: "register_owner",
            keys: keys,
            paymentAmount: paymentAmount ? paymentAmount : "1000000000",
            runtimeArgs,
            cb: (deployHash) => { },
            ttl: ttl ? ttl : DEFAULT_TTL,
        });
    }
    async mintOfficial({ keys, tokenOwner, mintId, metadataJson, paymentAmount, ttl }) {
        // Owner input is accountHash
        // tokenOwner = tokenOwner.startsWith("account-hash-")
        //     ? tokenOwner.slice(13)
        //     : tokenOwner;


        // let ownerAccountHashByte = Uint8Array.from(
        //     Buffer.from(tokenOwner, 'hex'),
        // )


        const ownerKey = createRecipientAddress(CLPublicKey.fromHex(tokenOwner))
        let hashesMap = ["32"]
        let a = CLValueParsers.toBytes(ownerKey)
        console.log("a ", a)

        let token_metadata = CLValueBuilder.list(metadataJson.map(id => CLValueBuilder.string(id)))
        let hashes = CLValueBuilder.list(hashesMap.map(hash => CLValueBuilder.string(hash)))

        // let token_metadata = new CLString(JSON.stringify(metadataJson))
        let runtimeArgs = {};
        runtimeArgs = RuntimeArgs.fromMap({
            token_owner: ownerKey,
            token_meta_datas: token_metadata,
            token_hashes: hashes,
            // mint_id: CLValueBuilder.string(mintId)
        });

        console.log("before")
        return await this.contractClient.contractCall({
            entryPoint: "mint",
            keys: keys,
            paymentAmount: paymentAmount ? paymentAmount : "22000000000",
            runtimeArgs,
            cb: (deployHash) => { },
            ttl: ttl ? ttl : DEFAULT_TTL,
        });
    }
    async mintBox({ keys, tokenOwner, metadataJson, paymentAmount, ttl }) {

        const ownerKey = createRecipientAddress(CLPublicKey.fromHex(tokenOwner))
        // let token_metadata = CLValueBuilder.list(metadataJson.map(id => CLValueBuilder.string(id)))
        // let hashes = CLValueBuilder.list(hashesMap.map(hash => CLValueBuilder.string(hash)))

        let token_metadata = new CLString(JSON.stringify(metadataJson))
        let runtimeArgs = {};
        runtimeArgs = RuntimeArgs.fromMap({
            token_owner: ownerKey,
            token_meta_data: token_metadata,
        });

        console.log("before")
        return await this.contractClient.contractCall({
            entryPoint: "mint",
            keys: keys,
            paymentAmount: paymentAmount ? paymentAmount : "22000000000",
            runtimeArgs,
            cb: (deployHash) => { },
            ttl: ttl ? ttl : DEFAULT_TTL,
        });
    }
    async setWhitelist({ keys, whitelistedUsers, paymentAmount, ttl }) {
        let addressesWhitelistArr = whitelistedUsers.map((e) => CLValueBuilder.string(e));
        let arr = []
        for (let i = 0; i < addressesWhitelistArr.length; i++) {
            whitelistedUsers[i] = CLPublicKey.fromHex(whitelistedUsers[i])
            whitelistedUsers[i] = createRecipientAddress(whitelistedUsers[i])
            arr.push(whitelistedUsers[i])

        }
        console.log("ARR length: ", arr.length)
        // console.log("arr: ", arr)

        let runtimeArgs = {};
        runtimeArgs = RuntimeArgs.fromMap({
            whitelisted_users: CLValueBuilder.list(arr),
            whitelisted_users1: CLValueBuilder.string(arr.join("-")),
            whitelisted_users2: CLValueBuilder.string(arr.join("-")),
        });

        console.log("before")
        return await this.contractClient.contractCall({
            entryPoint: "set_whitelist",
            keys: keys,
            paymentAmount: paymentAmount ? paymentAmount : "6000000000",
            runtimeArgs,
            cb: (deployHash) => { },
            ttl: ttl ? ttl : DEFAULT_TTL,
        });
    }

    // test approve to claim
    async approveToClaim({ keys, tokenOwner, mintId, metadataJson, paymentAmount, ttl }) {

        const ownerKey = createRecipientAddress(CLPublicKey.fromHex(tokenOwner))
        let hashesMap = ["64"]

        let token_metadata = CLValueBuilder.list(metadataJson.map(id => CLValueBuilder.string(id)))
        let hashes = CLValueBuilder.list(hashesMap.map(hash => CLValueBuilder.string(hash)))

        // let token_metadata = new CLString(JSON.stringify(metadataJson))
        let runtimeArgs = {};
        runtimeArgs = RuntimeArgs.fromMap({
            token_owner: ownerKey,
            token_meta_datas: token_metadata,
            mint_id: mintId,
            token_hashes: hashes
        });

        console.log("before")
        return await this.contractClient.contractCall({
            entryPoint: "approve_to_claim",
            keys: keys,
            paymentAmount: paymentAmount ? paymentAmount : "22000000000",
            runtimeArgs,
            cb: (deployHash) => { },
            ttl: ttl ? ttl : DEFAULT_TTL,
        });
    }

    async approveForAll(keys, operator, paymentAmount, ttl) {
        let key = createRecipientAddress(operator);
        let runtimeArgs = RuntimeArgs.fromMap({
            operator: key,
        });

        return await this.contractClient.contractCall({
            entryPoint: "set_approval_for_all",
            keys: keys,
            paymentAmount: paymentAmount ? paymentAmount : "1000000000",
            runtimeArgs,
            cb: (deployHash) => { },
            ttl: ttl ? ttl : DEFAULT_TTL,
        });
    }

    async burn(keys, tokenId, paymentAmount, ttl) {
        let identifierMode = await this.identifierMode();
        identifierMode = parseInt(identifierMode.toString());
        let runtimeArgs = {};
        if (identifierMode == 0) {
            runtimeArgs = RuntimeArgs.fromMap({
                token_id: CLValueBuilder.u64(parseInt(tokenId)),
            });
        } else {
            runtimeArgs = RuntimeArgs.fromMap({
                token_hash: CLValueBuilder.string(tokenId),
            });
        }

        return await this.contractClient.contractCall({
            entryPoint: "burn",
            keys: keys,
            paymentAmount: paymentAmount ? paymentAmount : "1000000000",
            runtimeArgs,
            cb: (deployHash) => { },
            ttl: ttl ? ttl : DEFAULT_TTL,
        });
    }

    async checkOperatorDictionaryKey(caller, operator) {
        try {
            let callerKey = createRecipientAddress(CLPublicKey.fromHex(caller))
            const contracthashbytearray = new CLByteArray(Uint8Array.from(Buffer.from(operator, 'hex')));
            const operatorKey = new CLKey(contracthashbytearray);
            let callerKeyBytes = CLValueParsers.toBytes(callerKey).val
            let operatorKeyBytes = CLValueParsers.toBytes(operatorKey).val
            let mix = Array.from(callerKeyBytes).concat(Array.from(operatorKeyBytes))
            let itemKeyArray = blake.blake2b(Buffer.from(mix), null, 32)
            let itemKey = Buffer.from(itemKeyArray).toString('hex')
            console.log("itemKey ", itemKey)
            console.log("ope ", this.namedKeys.operators)
            const result = await utils.contractDictionaryGetter(
                this.nodeAddress,
                itemKey,
                this.namedKeys.operators
            );
            return result
        } catch (e) {
            console.log(e)
        }


    }
    async transfer({ keys, source, recipient, tokenId, paymentAmount, ttl }) {
        let identifierMode = await this.identifierMode();
        identifierMode = parseInt(identifierMode.toString());
        let runtimeArgs = {};
        if (identifierMode == 0) {
            runtimeArgs = RuntimeArgs.fromMap({
                token_id: CLValueBuilder.u64(parseInt(tokenId)),
                source_key: createRecipientAddress(source),
                target_key: createRecipientAddress(recipient),
            });
        } else {
            runtimeArgs = RuntimeArgs.fromMap({
                token_hash: CLValueBuilder.string(tokenId),
                source_key: createRecipientAddress(source),
                target_key: createRecipientAddress(recipient),
            });
        }

        return await this.contractClient.contractCall({
            entryPoint: "transfer",
            keys: keys,
            paymentAmount: paymentAmount ? paymentAmount : "20000000000",
            runtimeArgs,
            cb: (deployHash) => { },
            ttl: ttl ? ttl : DEFAULT_TTL,
        });
    }
};

module.exports = CEP78;
