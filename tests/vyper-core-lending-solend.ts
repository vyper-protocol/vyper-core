import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import {
    createMintAndVault,
    createTokenAccount,
    getMintInfo,
    getTokenAccount,
} from "@project-serum/common";
import {
    ASSOCIATED_TOKEN_PROGRAM_ID,
    Token,
    TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import assert from "assert";
import { bn } from "./utils";
import {
    createTrancheConfigInput,
    createTranchesConfiguration,
    findTrancheConfig,
} from "./vyper-core-utils";
import {
    SolendReserveAsset,
    DEVNET_SOLEND_PROGRAM_ID,
    pythPrice,
    switchboardFeed,
} from "./solend/solend";

describe("vyper-core-lending-solend", () => {
    // Configure the client to use the local cluster.
    anchor.setProvider(anchor.Provider.env());

    //@ts-ignore
    const programVyperCoreLending = anchor.workspace
        .VyperCoreLending as Program<VyperCoreLending>;

    it("deposit on solend", async () => {
        // define input data
        const inputData = createTrancheConfigInput();
        const quantityToDeposit = 1000;

        // init SOLEND
        const solendInit = await initLendingMarkets();

        // mint reserve token to user wallet
        var userReserveTokenAccount = await createTokenAccount(
            programVyperCoreLending.provider,
            solendInit.reserveToken,
            programVyperCoreLending.provider.wallet.publicKey
        );

        const mintToTx = new anchor.web3.Transaction();
        mintToTx.add(
            Token.createMintToInstruction(
                TOKEN_PROGRAM_ID,
                solendInit.reserveToken,
                userReserveTokenAccount,
                programVyperCoreLending.provider.wallet.publicKey,
                [solendInit.owner],
                quantityToDeposit
            )
        );
        await programVyperCoreLending.provider.send(mintToTx, [
            solendInit.owner,
        ]);

        const userReserveTokenAccountInfo = await getTokenAccount(
            programVyperCoreLending.provider,
            userReserveTokenAccount
        );
        assert.equal(userReserveTokenAccountInfo.amount, quantityToDeposit);

        // initialize tranche config

        const {
            seniorTrancheMint,
            seniorTrancheMintBump,
            juniorTrancheMint,
            juniorTrancheMintBump,
        } = await createTranchesConfiguration(
            DEVNET_SOLEND_PROGRAM_ID,
            solendInit.reserveToken,
            programVyperCoreLending
        );

        const [trancheConfig, trancheConfigBump] = await findTrancheConfig(
            solendInit.reserveToken,
            seniorTrancheMint,
            juniorTrancheMint,
            programVyperCoreLending.programId
        );

        // vyper-core rpc: create tranche

        const tx = await programVyperCoreLending.rpc.createTranche(
            inputData,
            trancheConfigBump,
            seniorTrancheMintBump,
            juniorTrancheMintBump,
            {
                accounts: {
                    authority:
                        programVyperCoreLending.provider.wallet.publicKey,
                    trancheConfig,
                    mint: solendInit.reserveToken,
                    seniorTrancheMint: seniorTrancheMint,
                    juniorTrancheMint: juniorTrancheMint,
                    protocolProgram: DEVNET_SOLEND_PROGRAM_ID,
                    systemProgram: anchor.web3.SystemProgram.programId,
                    tokenProgram: TOKEN_PROGRAM_ID,
                    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
                    rent: anchor.web3.SYSVAR_RENT_PUBKEY,
                    clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
                },
            }
        );

        const seniorTrancheVault = await createTokenAccount(
            programVyperCoreLending.provider,
            seniorTrancheMint,
            programVyperCoreLending.provider.wallet.publicKey
        );
        const juniorTrancheVault = await createTokenAccount(
            programVyperCoreLending.provider,
            juniorTrancheMint,
            programVyperCoreLending.provider.wallet.publicKey
        );

        const vyperCollateralTokenAccount = await createTokenAccount(
            programVyperCoreLending.provider,
            new anchor.web3.PublicKey(
                solendInit.reserve.reserve.config.collateralMintAddress
            ),
            programVyperCoreLending.provider.wallet.publicKey
        );

        // deposit on lending protocol

        const seniorTrancheMintQuantity = 150;
        const juniorTrancheMintQuantity = 50;

        const tx2 = await programVyperCoreLending.rpc.deposit(
            bn(quantityToDeposit),
            [bn(seniorTrancheMintQuantity), bn(juniorTrancheMintQuantity)],
            {
                accounts: {
                    authority:
                        programVyperCoreLending.provider.wallet.publicKey,
                    trancheConfig,
                    reserveToken: solendInit.reserveToken,
                    sourceLiquidity: userReserveTokenAccount,

                    reserveLiquiditySupply:
                        solendInit.reserve.accounts.liquiditySupply,
                    destinationCollateralAccount: vyperCollateralTokenAccount,
                    collateralMint:
                        solendInit.reserve.reserve.config.collateralMintAddress,
                    protocolState: new anchor.web3.PublicKey(
                        solendInit.reserve.reserve.config.address
                    ),
                    lendingMarketAccount: solendInit.marketKeypair.publicKey,
                    lendingMarketAuthority:
                        solendInit.reserve.accounts.marketAuthority,
                    pythReserveLiquidityOracle: pythPrice,
                    switchboardReserveLiquidityOracle: switchboardFeed,

                    seniorTrancheMint,
                    seniorTrancheVault,

                    juniorTrancheMint,
                    juniorTrancheVault,

                    protocolProgram: DEVNET_SOLEND_PROGRAM_ID,
                    lendingProgram: DEVNET_SOLEND_PROGRAM_ID,
                    systemProgram: anchor.web3.SystemProgram.programId,
                    tokenProgram: TOKEN_PROGRAM_ID,
                    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
                    rent: anchor.web3.SYSVAR_RENT_PUBKEY,
                    clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
                },
            }
        );
        console.log("deposit tx: " + tx2);

        const account =
            await programVyperCoreLending.account.trancheConfig.fetch(
                trancheConfig
            );

        assert.equal(
            account.depositedQuantity
                .map((c) => c.toNumber())
                .reduce((a: number, b: number): number => {
                    return a + b;
                }, 0),
            quantityToDeposit
        );
        assert.deepEqual(account.interestSplit, inputData.interestSplit);
        assert.deepEqual(account.capitalSplit, inputData.capitalSplit);

        const userReserveTokenAccountInto = await getTokenAccount(
            programVyperCoreLending.provider,
            userReserveTokenAccount
        );
        assert.equal(
            userReserveTokenAccountInto.mint.toBase58(),
            solendInit.reserveToken.toBase58()
        );
        assert.equal(userReserveTokenAccountInto.amount.toNumber(), 0);

        const vyperCollateralTokenAccountInfo = await getTokenAccount(
            programVyperCoreLending.provider,
            vyperCollateralTokenAccount
        );
        assert.equal(
            vyperCollateralTokenAccountInfo.mint.toBase58(),
            solendInit.reserve.accounts.collateralMint.toBase58()
        );
        assert.equal(
            vyperCollateralTokenAccountInfo.amount.toNumber() > 0,
            true
        );

        const seniorTrancheMintInfo = await getMintInfo(
            programVyperCoreLending.provider,
            seniorTrancheMint
        );
        assert.equal(seniorTrancheMintInfo.decimals, 0);
        assert.equal(
            seniorTrancheMintInfo.supply.toNumber(),
            seniorTrancheMintQuantity
        );

        const seniorTrancheVaultInto = await getTokenAccount(
            programVyperCoreLending.provider,
            seniorTrancheVault
        );
        assert.equal(seniorTrancheVaultInto.amount, seniorTrancheMintQuantity);

        const juniorTrancheMintInfo = await getMintInfo(
            programVyperCoreLending.provider,
            juniorTrancheMint
        );
        assert.equal(juniorTrancheMintInfo.decimals, 0);
        assert.equal(
            juniorTrancheMintInfo.supply.toNumber(),
            juniorTrancheMintQuantity
        );

        const juniorTrancheVaultInto = await getTokenAccount(
            programVyperCoreLending.provider,
            juniorTrancheVault
        );
        assert.equal(juniorTrancheVaultInto.amount, juniorTrancheMintQuantity);
    });

    it("deposit to and redeem everything from solend", async () => {
        // define input data
        const inputData = createTrancheConfigInput();
        const quantityToDeposit = 100000;

        // init SOLEND
        const solendInit = await initLendingMarkets();

        console.log(
            "collateralMintAddress: " +
                solendInit.reserve.reserve.config.collateralMintAddress
        );
        console.log(
            "mintAddress: " + solendInit.reserve.reserve.config.mintAddress
        );

        // mint reserve token to user wallet
        var userReserveTokenAccount = await createTokenAccount(
            programVyperCoreLending.provider,
            solendInit.reserveToken,
            programVyperCoreLending.provider.wallet.publicKey
        );

        const mintToTx = new anchor.web3.Transaction();
        mintToTx.add(
            Token.createMintToInstruction(
                TOKEN_PROGRAM_ID,
                solendInit.reserveToken,
                userReserveTokenAccount,
                programVyperCoreLending.provider.wallet.publicKey,
                [solendInit.owner],
                quantityToDeposit
            )
        );
        await programVyperCoreLending.provider.send(mintToTx, [
            solendInit.owner,
        ]);

        const userReserveTokenAccountInfo = await getTokenAccount(
            programVyperCoreLending.provider,
            userReserveTokenAccount
        );
        assert.equal(userReserveTokenAccountInfo.amount, quantityToDeposit);

        // initialize tranche config

        const {
            seniorTrancheMint,
            seniorTrancheMintBump,
            juniorTrancheMint,
            juniorTrancheMintBump,
        } = await createTranchesConfiguration(
            DEVNET_SOLEND_PROGRAM_ID,
            solendInit.reserveToken,
            programVyperCoreLending
        );

        const [trancheConfig, trancheConfigBump] = await findTrancheConfig(
            solendInit.reserveToken,
            seniorTrancheMint,
            juniorTrancheMint,
            programVyperCoreLending.programId
        );

        // vyper-core rpc: create tranche

        const tx1 = await programVyperCoreLending.rpc.createTranche(
            inputData,
            trancheConfigBump,
            seniorTrancheMintBump,
            juniorTrancheMintBump,
            {
                accounts: {
                    authority:
                        programVyperCoreLending.provider.wallet.publicKey,
                    trancheConfig,
                    mint: solendInit.reserveToken,
                    seniorTrancheMint: seniorTrancheMint,
                    juniorTrancheMint: juniorTrancheMint,
                    protocolProgram: DEVNET_SOLEND_PROGRAM_ID,
                    systemProgram: anchor.web3.SystemProgram.programId,
                    tokenProgram: TOKEN_PROGRAM_ID,
                    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
                    rent: anchor.web3.SYSVAR_RENT_PUBKEY,
                    clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
                },
            }
        );
        console.log("create tranche tx: " + tx1);

        const seniorTrancheVault = await createTokenAccount(
            programVyperCoreLending.provider,
            seniorTrancheMint,
            programVyperCoreLending.provider.wallet.publicKey
        );
        const juniorTrancheVault = await createTokenAccount(
            programVyperCoreLending.provider,
            juniorTrancheMint,
            programVyperCoreLending.provider.wallet.publicKey
        );

        const vyperCollateralTokenAccount = await createTokenAccount(
            programVyperCoreLending.provider,
            new anchor.web3.PublicKey(
                solendInit.reserve.reserve.config.collateralMintAddress
            ),
            programVyperCoreLending.provider.wallet.publicKey
        );

        // vyper-core rpc: deposit on lending protocol

        const seniorTrancheMintQuantity = 150;
        const juniorTrancheMintQuantity = 50;

        const tx2 = await programVyperCoreLending.rpc.deposit(
            bn(quantityToDeposit),
            [bn(seniorTrancheMintQuantity), bn(juniorTrancheMintQuantity)],
            {
                accounts: {
                    authority:
                        programVyperCoreLending.provider.wallet.publicKey,
                    trancheConfig,
                    reserveToken: solendInit.reserveToken,
                    sourceLiquidity: userReserveTokenAccount,

                    reserveLiquiditySupply:
                        solendInit.reserve.accounts.liquiditySupply,
                    destinationCollateralAccount: vyperCollateralTokenAccount,
                    collateralMint:
                        solendInit.reserve.reserve.config.collateralMintAddress,
                    protocolState: new anchor.web3.PublicKey(
                        solendInit.reserve.reserve.config.address
                    ),
                    lendingMarketAccount: solendInit.marketKeypair.publicKey,
                    lendingMarketAuthority:
                        solendInit.reserve.accounts.marketAuthority,
                    pythReserveLiquidityOracle: pythPrice,
                    switchboardReserveLiquidityOracle: switchboardFeed,

                    seniorTrancheMint,
                    seniorTrancheVault,

                    juniorTrancheMint,
                    juniorTrancheVault,

                    protocolProgram: DEVNET_SOLEND_PROGRAM_ID,
                    lendingProgram: DEVNET_SOLEND_PROGRAM_ID,
                    systemProgram: anchor.web3.SystemProgram.programId,
                    tokenProgram: TOKEN_PROGRAM_ID,
                    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
                    rent: anchor.web3.SYSVAR_RENT_PUBKEY,
                    clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
                },
            }
        );
        console.log("deposit tx: " + tx2);

        // vyper-core rpc:

        const tx3 = await programVyperCoreLending.rpc.redeem(
            [bn(seniorTrancheMintQuantity), bn(juniorTrancheMintQuantity)],
            {
                accounts: {
                    authority:
                        programVyperCoreLending.provider.wallet.publicKey,
                    trancheConfig,
                    reserveToken: solendInit.reserveToken,
                    destinationLiquidity: userReserveTokenAccount,

                    reserveLiquiditySupply:
                        solendInit.reserve.accounts.liquiditySupply,
                    sourceCollateralAccount: vyperCollateralTokenAccount,
                    collateralMint:
                        solendInit.reserve.reserve.config.collateralMintAddress,
                    protocolState: new anchor.web3.PublicKey(
                        solendInit.reserve.reserve.config.address
                    ),
                    lendingMarketAccount: solendInit.marketKeypair.publicKey,
                    lendingMarketAuthority:
                        solendInit.reserve.accounts.marketAuthority,
                    pythReserveLiquidityOracle: pythPrice,
                    switchboardReserveLiquidityOracle: switchboardFeed,

                    seniorTrancheMint,
                    seniorTrancheVault,

                    juniorTrancheMint,
                    juniorTrancheVault,

                    protocolProgram: DEVNET_SOLEND_PROGRAM_ID,
                    lendingProgram: DEVNET_SOLEND_PROGRAM_ID,
                    systemProgram: anchor.web3.SystemProgram.programId,
                    tokenProgram: TOKEN_PROGRAM_ID,
                    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
                    rent: anchor.web3.SYSVAR_RENT_PUBKEY,
                    clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
                },
            }
        );
        console.log("redeem tx: " + tx3);

        const account =
            await programVyperCoreLending.account.trancheConfig.fetch(
                trancheConfig
            );

        assert.equal(
            account.depositedQuantity
                .map((c) => c.toNumber())
                .reduce((a: number, b: number): number => {
                    return a + b;
                }, 0),
            quantityToDeposit
        );
        assert.deepEqual(account.interestSplit, inputData.interestSplit);
        assert.deepEqual(account.capitalSplit, inputData.capitalSplit);

        const userReserveTokenAccountInto = await getTokenAccount(
            programVyperCoreLending.provider,
            userReserveTokenAccount
        );
        assert.equal(
            userReserveTokenAccountInto.mint.toBase58(),
            solendInit.reserveToken.toBase58()
        );
        assert.equal(
            userReserveTokenAccountInto.amount.toNumber(),
            quantityToDeposit
        );

        const vyperCollateralTokenAccountInfo = await getTokenAccount(
            programVyperCoreLending.provider,
            vyperCollateralTokenAccount
        );
        assert.equal(
            vyperCollateralTokenAccountInfo.mint.toBase58(),
            solendInit.reserve.accounts.collateralMint.toBase58()
        );
        assert.equal(vyperCollateralTokenAccountInfo.amount.toNumber(), 0);

        const seniorTrancheMintInfo = await getMintInfo(
            programVyperCoreLending.provider,
            seniorTrancheMint
        );
        assert.equal(seniorTrancheMintInfo.decimals, 0);
        assert.equal(seniorTrancheMintInfo.supply.toNumber(), 0);

        const seniorTrancheVaultInto = await getTokenAccount(
            programVyperCoreLending.provider,
            seniorTrancheVault
        );
        assert.equal(seniorTrancheVaultInto.amount, 0);

        const juniorTrancheMintInfo = await getMintInfo(
            programVyperCoreLending.provider,
            juniorTrancheMint
        );
        assert.equal(juniorTrancheMintInfo.decimals, 0);
        assert.equal(juniorTrancheMintInfo.supply.toNumber(), 0);

        const juniorTrancheVaultInto = await getTokenAccount(
            programVyperCoreLending.provider,
            juniorTrancheVault
        );
        assert.equal(juniorTrancheVaultInto.amount, 0);
    });

    interface InitLendingMarketResult {
        reserve: SolendReserveAsset;
        marketKeypair: anchor.web3.Keypair;
        owner: anchor.web3.Keypair;
        reserveToken: anchor.web3.PublicKey;
        ownerReserveTokenAccount: anchor.web3.PublicKey;
    }

    async function initLendingMarkets(): Promise<InitLendingMarketResult> {
        const initialReserveAmount = 100;
        const solendOwner = anchor.web3.Keypair.generate();
        const [reserveToken, ownerReserveTokenAccount] =
            await createMintAndVault(
                programVyperCoreLending.provider,
                bn(3 * initialReserveAmount),
                solendOwner.publicKey,
                2
            );

        const pythProduct = new anchor.web3.PublicKey(
            "ALP8SdU9oARYVLgLR7LrqMNCYBnhtnQz1cj6bwgwQmgj"
        );

        const pythProgram = new anchor.web3.PublicKey(
            "FsJ3A3u2vn5cTVofAjvy6y5kwABJAqYWpe4975bi2epH"
        );
        const switchboardProgram = new anchor.web3.PublicKey(
            "DtmE9D2CSB4L5D6A15mraeEjrGMm6auWVzgaD8hK2tZM"
        );

        const [solendReserve, marketKeypair] =
            await SolendReserveAsset.initialize(
                programVyperCoreLending.provider,
                solendOwner,
                // @ts-ignore
                programVyperCoreLending.provider.wallet,
                reserveToken,
                pythProgram,
                switchboardProgram,
                pythProduct,
                pythPrice,
                switchboardFeed,
                ownerReserveTokenAccount,
                initialReserveAmount
            );

        return {
            owner: solendOwner,
            marketKeypair,
            ownerReserveTokenAccount,
            reserveToken,
            reserve: solendReserve,
        };
    }
});
