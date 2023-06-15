const {
    CasperClient,
    CLPublicKey,
    Keys,
    CLKey,
    CLValueParsers,
    CLByteArray,
    CasperServiceByJsonRPC,
} = require("casper-js-sdk");
const blake = require("blakejs")

const {
    utils,
    helpers,
    CasperContractClient,
} = require("casper-js-client-helper");


const { setClient, contractSimpleGetter, createRecipientAddress } = helpers;


const Utils = {
    parseTokenMeta: (str) =>
        str.split(",").map((s) => {
            const map = s.split("");
            return [map[0], map[1]];
        }),
    sleep: (ms) => {
        return new Promise((resolve) => setTimeout(resolve, ms));
    },

    /**
    * Returns a set ECC key pairs - one for each NCTL user account.
    * @param {String} pathToUsers - Path to NCTL user directories.
    * @return {Array} An array of assymmetric keys.
    */
    getKeyPairOfUserSet: (pathToUsers) => {
        return [1, 2, 3, 4, 5].map((userID) => {
            return Keys.Ed25519.parseKeyFiles(
                `${pathToUsers}/user-${userID}/public_key.pem`,
                `${pathToUsers}/user-${userID}/secret_key.pem`
            );
        });
    },

    getDeploy: async (NODE_URL, deployHash) => {
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
                await Utils.sleep(1000);
                continue;
            }
        }
        throw Error("Timeout after " + i + "s.Something’s wrong");
    },

    getAccountInfo: async (nodeAddress, publicKey) => {
        const client = new CasperServiceByJsonRPC(nodeAddress);
        const stateRootHash = await client.getStateRootHash();
        const accountHash = publicKey.toAccountHashStr();
        const blockState = await client.getBlockState(
            stateRootHash,
            accountHash,


        );
        return blockState.Account;
    },

    /**
    * Returns a value under an on-chain account’s storage.
    * @param accountInfo - On-chain account’s info.
    * @param namedKey - A named key associated with an on-chain account.
    */
    getAccountNamedKeyValue: (accountInfo, namedKey) => {
        const found = accountInfo.namedKeys.find((i) => i.name === namedKey);
        if (found) {
            return found.key;
        }
        return undefined;
    },
    getOperatorDictionaryKey: (caller, operator) => {
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


};
module.exports = Utils;
