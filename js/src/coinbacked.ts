import * as solanaWeb3 from "@solana/web3.js";
import {Buffer} from "buffer";

// interfaces
export interface BackingAccount
{
    accountInfo: solanaWeb3.AccountInfo<Buffer>,
    mintKey: solanaWeb3.PublicKey,
    rentExemptionLamports: bigint,
    bump: number
}

// core entry point factory
export default class coinbackedWeb3
{
    static api(connection: solanaWeb3.Connection):Api
    {
        return new Api(connection);
    }

    static instructions(api: Api): Instructions
    {
        return new Instructions(api);
    }

    public static PROGRAM_ID: solanaWeb3.PublicKey = new solanaWeb3.PublicKey("B91LvPYXAo3KVNFbSXkWJWunVtXMV5irzdWCqPxJfMR7");
    public static SEED_COINBACKED: Uint8Array = new Uint8Array([67, 79, 73, 78, 66, 65, 67, 75, 69, 68]); // COINBACKED
    public static SEED_COINBACKED_TREASURY: Uint8Array = new Uint8Array([67, 79, 73, 78, 66, 65, 67, 75, 69, 68, 45, 84, 82, 69, 65, 83, 85, 82, 89]); // COINBACKED-TREASURY
}

// implementation API
export class Api
{
    private _connection?: solanaWeb3.Connection;
    private _coinbackedProgramPubkey: solanaWeb3.PublicKey = coinbackedWeb3.PROGRAM_ID;
    private _coinbackedSeed: Uint8Array = coinbackedWeb3.SEED_COINBACKED;
    private _coinbackedTreasurySeed: Uint8Array = coinbackedWeb3.SEED_COINBACKED_TREASURY;
    private _coinbackedAccountDataLength: number = 41;

    constructor(connection: solanaWeb3.Connection)
    {
        this._connection = connection;
    }

    getBackingAccount(mintKey: solanaWeb3.PublicKey):Promise<BackingAccount>
    {
        return new Promise((resolve, reject) =>
        {
            let backingPDA = this.getBackingAccountAddress(mintKey);
            this._connection?.getAccountInfo(backingPDA.key, 'confirmed').then((accountInfo) =>
            {
                if ((!accountInfo?.owner.equals(this._coinbackedProgramPubkey)) ||
                    (accountInfo?.lamports == 0) ||
                    (accountInfo?.data.byteLength != this._coinbackedAccountDataLength))
                {
                    reject("Invalid or no backing account.");
                }                
                
                let result:BackingAccount = 
                {
                    accountInfo: accountInfo,
                    mintKey: new solanaWeb3.PublicKey(accountInfo?.data.slice(0,32)),
                    rentExemptionLamports: accountInfo?.data.slice(32,40).readBigUInt64LE(0),
                    bump: accountInfo?.data.slice(40,41).readUint8(0),
                };
                
                if (result.bump != backingPDA.bump)
                {
                    reject("Invalid or no backing account.");
                }

                if (!mintKey.equals(result.mintKey))
                {
                    reject("Backing account does not belong to mint.")
                }

                resolve(result);

            }).catch((reason) => 
            {
                reject(reason);
            });
        });
    }

    isMintBacked(mintKey: solanaWeb3.PublicKey):Promise<boolean>
    {
        return new Promise((resolve, reject) => 
        {
            this.getBackingAccount(mintKey).then((result) =>
            {
                resolve(true)
            }).catch((e) =>
            {
                resolve(false)
            });
        });

    }

    getBackingLamports(mintKey:solanaWeb3.PublicKey):Promise<bigint>
    {
        return new Promise((resolve, reject) =>
        {
            this.getBackingAccount(mintKey).then((result) =>
            {
                resolve(BigInt(result.accountInfo.lamports))
            }).catch((e) =>
            {
                reject("Could not retrieve backing account.")
            });     
        });
    }

    getPayoutInLamports(mintKey:solanaWeb3.PublicKey, tokenAmount: bigint):Promise<bigint>
    {
        return new Promise((resolve, reject) => 
        {
            var miscResults = {
                supply: BigInt(0),
                backingLamports: BigInt(0)
            };

            this._getMintAccountInfos(mintKey).then((accountInfos) => 
            {
                miscResults.supply = accountInfos.supply;
                return this.getBackingLamports(mintKey);

            }).then((lamports) => 
            {
                miscResults.backingLamports = lamports;
                return this.getBackingAccount(mintKey);

            }).then((backingAccount) =>
            {
                // calculate per unit payout based on supply
                // token_amount * (backing_lamports - backing_rent_excemption) / supply
                resolve(tokenAmount * (miscResults.backingLamports - backingAccount.rentExemptionLamports) / miscResults.supply);
            }).catch((e) => 
            {
                reject(e);
            });
        });
    }
    
    tokenAmountOneUnit(decimals: number): bigint
    {
        return BigInt(Math.pow(10, decimals));
    }

    tokenAmountUI(tokenAmount: bigint, decimals: number): string
    {
        if (decimals == 0)
        {
            return tokenAmount.toString();
        }
        else
        {
            BigDecimal.decimals = decimals;
            return new BigDecimal(tokenAmount).divide(new BigDecimal(this.tokenAmountOneUnit(decimals))).toString();
        }
    }

    solanaAmountUI(lamports: bigint, visibleDigits?: number): string
    {
        if (typeof visibleDigits !== 'undefined')
        {
            let completeString = this.tokenAmountUI(lamports, 9);
            let commaPos = completeString.indexOf('.');

            return completeString.substring(0, commaPos+visibleDigits+1);
        }
        return this.tokenAmountUI(lamports, 9);
    }

    getBackingAccountAddress(mintKey: solanaWeb3.PublicKey): {key: solanaWeb3.PublicKey, bump: number}
    {
        let result = solanaWeb3.PublicKey.findProgramAddressSync([mintKey.toBytes(), this._coinbackedProgramPubkey.toBytes(), this._coinbackedSeed], this._coinbackedProgramPubkey);
        return {key: result[0], bump: result[1]};
    }

    getTreasuryAccountAddress(): {key: solanaWeb3.PublicKey, bump: number}
    {
        let result = solanaWeb3.PublicKey.findProgramAddressSync([this._coinbackedProgramPubkey.toBytes(), this._coinbackedTreasurySeed], this._coinbackedProgramPubkey);
        return ({key: result[0], bump: result[1]});
    }

    getTokenAccountAddress(mintKey: solanaWeb3.PublicKey, owner: solanaWeb3.PublicKey): Promise<solanaWeb3.PublicKey>
    {
        return new Promise((resolve, reject) => 
        {
            this._connection.getTokenAccountsByOwner(owner, {mint: mintKey}).then((results) =>
            {
                if (results.value.length == 0)
                {
                    reject("No token account for owner found for this mint.");
                }

                results.value.forEach((item) =>
                {
                    let tokenAccount = this._getTokenAccountInfos(item.account);
                    if (tokenAccount.amount > 0)
                    {
                        resolve(item.pubkey)
                    }
                });

                reject("No token account for owner found for this mint.");
            });
        });
    }

    private _getTokenAccountInfos(accountInfo: solanaWeb3.AccountInfo<Buffer>): {mint: solanaWeb3.PublicKey, owner: solanaWeb3.PublicKey, amount: bigint}
    {
        if (accountInfo.data.length < 72)
        {
            return {mint: null, owner: null, amount: BigInt(0)};
        }

        // mint account layout:
        // https://github.com/solana-labs/solana-program-library/blob/08d9999f997a8bf38719679be9d572f119d0d960/token/program/src/state.rs#L86-L105
        return {
            mint:  new solanaWeb3.PublicKey(accountInfo?.data.slice(0,32)),
            owner: new solanaWeb3.PublicKey(accountInfo?.data.slice(32,64)),
            amount: accountInfo?.data.slice(64,72).readBigUInt64LE(0)
        };
    }

    private _getMintAccountInfos(mintKey: solanaWeb3.PublicKey): Promise<{supply: bigint, isSupplyFixed: boolean, decimals: number}>
    {
        return new Promise((resolve, reject) => 
        {            
            this._connection?.getAccountInfo(mintKey, 'confirmed').then((accountInfo) => 
            {
                if (!accountInfo.owner)
                {
                    reject("Not a proper mint account.");
                }

                if (accountInfo.data.length < 82)
                {
                    reject("Invalid account, data length missmatch.");
                }

                // mint account layout:
                // https://github.com/solana-labs/solana-program-library/blob/08d9999f997a8bf38719679be9d572f119d0d960/token/program/src/state.rs#L16-L29

                let mintAuth = new solanaWeb3.PublicKey(accountInfo?.data.slice(4,36)); // COptional type (4 byte padding)
                let supply = accountInfo?.data.slice(36,44).readBigUInt64LE(0);
                let decimals = accountInfo?.data.slice(44, 45).readUint8(0);

                resolve({supply: supply, isSupplyFixed: (mintAuth == null), decimals: decimals});

            }).catch((e) =>
            {
                reject(e);
            });

        });
    }
    
}

// instructions API - contains all relevant transactions to interact with program
export class Instructions
{
    private _api: Api;

    private static _OPERATION_CREAT_BACKING_ACCOUNT:number = 0;
    private static _OPERATION_VALIDATE_ACCOUNT:number = 1;
    private static _OPERATION_ADD_TO_BACKING_ACCOUNT:number = 2;


    constructor(api: Api)
    {
        this._api = api;
    }

    creationInstructions(mintKey: solanaWeb3.PublicKey, owner: solanaWeb3.PublicKey, backingLamports: bigint): Promise<[solanaWeb3.TransactionInstruction]>
    {
        return new Promise((resolve, reject) => 
        {
            // get required account keys from mint key
            let backingAccountKey = this._api.getBackingAccountAddress(mintKey).key;
            let treasuryAccountKey = this._api.getTreasuryAccountAddress().key;
            this._api.getTokenAccountAddress(mintKey, owner).then((tokenAccountKey) =>
            {
                const transactionData = Buffer.alloc(137);
                transactionData.writeInt8(Instructions._OPERATION_CREAT_BACKING_ACCOUNT, 0);
                transactionData.writeBigInt64LE(backingLamports, 1);
                /* TODO ToS missing */

                resolve([new solanaWeb3.TransactionInstruction(
                    {
                        keys: [
                            {pubkey: owner, isSigner: true, isWritable: true},
                            {pubkey: mintKey, isSigner: false, isWritable: false},
                            {pubkey: tokenAccountKey, isSigner: false, isWritable: false},
                            {pubkey: backingAccountKey, isSigner: false, isWritable: true},
                            {pubkey: treasuryAccountKey, isSigner: false, isWritable: true},

                            {pubkey: solanaWeb3.SystemProgram.programId, isSigner: false, isWritable: false},
                            {pubkey: solanaWeb3.SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false}
                        ],
                        programId: coinbackedWeb3.PROGRAM_ID,
                        data: transactionData,
                    })]);
            }).catch((e) =>
            {
                reject("Could not create instruction.")
            });
        });
    }

    
    addBackingLamportsInstructions(mintKey: solanaWeb3.PublicKey, feePayer: solanaWeb3.PublicKey, backingLamports: bigint): Promise<[solanaWeb3.TransactionInstruction]>
    {
        return new Promise((resolve, reject) =>
        {
            // get required account keys from mint key
            let backingAccountKey = this._api.getBackingAccountAddress(mintKey).key;
            let treasuryAccountKey = this._api.getTreasuryAccountAddress().key;

            // build transaction data
            const transactionData = Buffer.alloc(137);
            transactionData.writeInt8(Instructions._OPERATION_ADD_TO_BACKING_ACCOUNT, 0);
            transactionData.writeBigInt64LE(backingLamports, 1);
            /* TODO ToS missing */

            // build an return transaction
            resolve([new solanaWeb3.TransactionInstruction({
                keys: [
                    {pubkey: feePayer, isSigner: true, isWritable: true},
                    {pubkey: mintKey, isSigner: false, isWritable: false},
                    {pubkey: backingAccountKey, isSigner: false, isWritable: true},
                    {pubkey: treasuryAccountKey, isSigner: false, isWritable: true},

                    {pubkey: solanaWeb3.SystemProgram.programId, isSigner: false, isWritable: false},
                ],
                programId: coinbackedWeb3.PROGRAM_ID,
                data: transactionData
            })]);
        });
    }
    

    burnInstructions()
    {}

    validationInstructions(mintKey: solanaWeb3.PublicKey, feePayer: solanaWeb3.PublicKey): Promise<[solanaWeb3.TransactionInstruction?]>
    {
        return new Promise((resolve, reject) => 
        {
            // get required account keys from mint key
            let backingAccountKey = this._api.getBackingAccountAddress(mintKey).key;
            let treasuryAccountKey = this._api.getTreasuryAccountAddress().key;

            // build and return transaction
            resolve([new solanaWeb3.TransactionInstruction(
                {
                    keys: [
                        {pubkey: feePayer, isSigner: true, isWritable: true},
                        {pubkey: mintKey, isSigner: false, isWritable: false},
                        {pubkey: backingAccountKey, isSigner: false, isWritable: false},
                        {pubkey: treasuryAccountKey, isSigner: false, isWritable: true},
                        {pubkey: solanaWeb3.SystemProgram.programId, isSigner: false, isWritable: false}
                    ],
                    programId: coinbackedWeb3.PROGRAM_ID,
                    data: Buffer.from([Instructions._OPERATION_VALIDATE_ACCOUNT])
                })]);
        });
    }
}

// helpers
class BigDecimal 
{
    // helper class to handle bigint division...

    bigint: bigint = BigInt(0);
    static decimals: number = 0;

    constructor(value) 
    {
        let [ints, decis] = String(value).split(".").concat("");
        decis = decis.padEnd(BigDecimal.decimals, "0");
        this.bigint = BigInt(ints + decis);
    }

    static fromBigInt(bigint) 
    {
        return Object.assign(Object.create(BigDecimal.prototype), { bigint });
    }

    divide(divisor) 
    { 
        return BigDecimal.fromBigInt(this.bigint * BigInt("1" + "0".repeat(BigDecimal.decimals)) / divisor.bigint);
    }

    toString() 
    {
        const s = this.bigint.toString().padStart(BigDecimal.decimals+1, "0");
        return s.slice(0, -BigDecimal.decimals) + "." + s.slice(-BigDecimal.decimals);
    }
}