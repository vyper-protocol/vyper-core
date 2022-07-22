import * as anchor from "@project-serum/anchor";
import { PublicKey } from "@solana/web3.js";
import { VyperCore } from "../../target/types/vyper_core";
import idlVyperCore from '../../target/idl/vyper_core.json';
import { TrancheConfig } from "./TrancheConfig";
import { SlotTracking } from "./SlotTracking";
import { LastUpdate } from "./LastUpdate";
import { ReserveFairValue } from "./ReserveFairValue";
import { TrancheData } from "./TrancheData";
import { TrancheFairValue } from "./TrancheFairValue";
import { IRedeemLogicPlugin} from "./plugins/redeemLogicPlugin/IReedeemLogicPlugin";
import { IRatePlugin } from "./plugins/ratePlugin/IRatePlugin";
import { HaltFlags } from "./HaltFlags";
import {UpdateTrancheConfigFlags} from "./UpdateTrancheConfigFlags"
import { OwnerRestrictedIxFlags } from "./OwnerRestrictedIxFlags";
import { InitializationData } from "./TrancheInitData";

export class Vyper {

    program: anchor.Program<VyperCore>;
    provider: anchor.AnchorProvider;
    trancheId: PublicKey;
    redeemLogicPlugin: IRedeemLogicPlugin;
    ratePlugin: IRatePlugin;
    seniorTrancheMint: PublicKey;
    juniorTrancheMint: PublicKey;
    trancheAuthority: PublicKey;
    reserveMint: PublicKey;
    reserve: PublicKey;

    static create(provider: anchor.AnchorProvider, vyperCoreId: PublicKey, redeemLogicPlugin?: IRedeemLogicPlugin, ratePlugin?: IRatePlugin): Vyper {
        const client = new Vyper();
        const program = new anchor.Program(idlVyperCore as any, vyperCoreId, provider) as anchor.Program<VyperCore>;
        client.program = program;
        client.provider = provider;

        if (redeemLogicPlugin) {
            client.redeemLogicPlugin = redeemLogicPlugin;
        }
        if (ratePlugin) {
            client.ratePlugin = ratePlugin;
        }
        
        return client;
    }

    async getTrancheConfiguration(trancheId?: PublicKey): Promise<TrancheConfig> {
        // if not supplied we take if from object
        if (!trancheId) {
            trancheId = this.trancheId
        }

        const trancheInfo = await this.program.account.trancheConfig.fetch(trancheId);

        const slotTrackingReserve = new SlotTracking(
            new LastUpdate(
                trancheInfo.trancheData.reserveFairValue['slotTracking']['lastUpdate']['slot'],
                trancheInfo.trancheData.reserveFairValue['slotTracking']['lastUpdate']['padding']
            ),
            trancheInfo.trancheData.reserveFairValue['slotTracking']['staleSlotThreshold']
        );

        const slotTrackingTranche = new SlotTracking(
            new LastUpdate(
                trancheInfo.trancheData.trancheFairValue['slotTracking']['lastUpdate']['slot'],
                trancheInfo.trancheData.trancheFairValue['slotTracking']['lastUpdate']['padding']
            ),
            trancheInfo.trancheData.trancheFairValue['slotTracking']['staleSlotThreshold']
        );

        const reserveFairValue = new ReserveFairValue(
            trancheInfo.trancheData.reserveFairValue['value'],
            slotTrackingReserve
        );

        const trancheFairValue = new TrancheFairValue(
            trancheInfo.trancheData.trancheFairValue['value'],
            slotTrackingTranche
        );

        const trancheData = new TrancheData(
            trancheInfo.trancheData.depositedQuantity.map((x) => x.toNumber()),
            trancheInfo.trancheData.feeToCollectQuantity.toNumber(),
            reserveFairValue,
            trancheFairValue,
            trancheInfo.trancheData.ownerRestrictedIx,
            trancheInfo.trancheData.haltFlags
        );

        const trancheConfig = new TrancheConfig(
            trancheInfo.reserveMint,
            trancheInfo.reserve,
            trancheData,
            trancheInfo.seniorTrancheMint,
            trancheInfo.juniorTrancheMint,
            trancheInfo.trancheAuthority,
            trancheInfo.authoritySeed,
            trancheInfo.authorityBump,
            trancheInfo.owner,
            trancheInfo.rateProgram,
            trancheInfo.rateProgramState,
            trancheInfo.redeemLogicProgram,
            trancheInfo.redeemLogicProgramState,
            trancheInfo.version,
            trancheInfo.createdAt.toNumber()
        );
        return trancheConfig;
    }

    async updateTrancheConfig(
        bitmask: UpdateTrancheConfigFlags,
        haltFlags: HaltFlags,
        ownerRestrictedIxs: OwnerRestrictedIxFlags,
        reserveFairValueStaleSlotThreshold: number,
        trancheFairValueStaleSlotThreshold: number,
    ) {
        await this.program.methods.updateTrancheData({
            bitmask: bitmask,
            haltFlags: haltFlags,
            ownerRestrictedIxs: ownerRestrictedIxs,
            reserveFairValueStaleSlotThreshold: new anchor.BN(reserveFairValueStaleSlotThreshold),
            trancheFairValueStaleSlotThreshold: new anchor.BN(trancheFairValueStaleSlotThreshold),
        }).accounts({
            owner: this.provider.wallet.publicKey,
            trancheConfig: this.trancheId
        }).rpc();
    }

    async refreshTrancheFairValue(trancheId?: PublicKey) {
        
        if(!trancheId) {
            trancheId = this.trancheId;
        }

        const trancheConfig = await this.getTrancheConfiguration(trancheId);
        await this.program.methods
            .refreshTrancheFairValue()
            .accounts({
                signer: this.provider.wallet.publicKey,
                trancheConfig: trancheId,
                seniorTrancheMint: trancheConfig.seniorTrancheMint,
                juniorTrancheMint: trancheConfig.juniorTrancheMint,
                rateProgramState: this.ratePlugin.rateStateId,
                redeemLogicProgram: this.redeemLogicPlugin.getProgramId(),
                redeemLogicProgramState: this.redeemLogicPlugin.redeemLogicStateId,
            })
            .rpc();
    }

    async getRefreshTrancheFairValueIX(trancheId?: PublicKey): Promise<anchor.web3.TransactionInstruction> {

        if(!trancheId) {
            trancheId = this.trancheId;
        }
        const trancheConfig = await this.getTrancheConfiguration(trancheId);
        return await this.program.methods
            .refreshTrancheFairValue()
            .accounts({
                signer: this.provider.wallet.publicKey,
                trancheConfig: trancheId,
                seniorTrancheMint: trancheConfig.seniorTrancheMint,
                juniorTrancheMint: trancheConfig.juniorTrancheMint,
                rateProgramState: this.ratePlugin.rateStateId,
                redeemLogicProgram: this.redeemLogicPlugin.getProgramId(),
                redeemLogicProgramState: this.redeemLogicPlugin.redeemLogicStateId
            })
            .instruction();
    }

    async initialize(
        initData: InitializationData,
        reserveMint: PublicKey,
        redeemLogicPlugin?: IRedeemLogicPlugin, 
        ratePlugin?: IRatePlugin,
        owner?: PublicKey
    ) {
        
        if(!ratePlugin) {
            ratePlugin = this.ratePlugin;
        }
        
        if(!redeemLogicPlugin) {
            redeemLogicPlugin = this.redeemLogicPlugin;
        }
        
        const juniorTrancheMint = anchor.web3.Keypair.generate();
        const seniorTrancheMint = anchor.web3.Keypair.generate();
        const trancheConfig = anchor.web3.Keypair.generate();
        const [trancheAuthority] = await anchor.web3.PublicKey.findProgramAddress(
            [trancheConfig.publicKey.toBuffer(), anchor.utils.bytes.utf8.encode("authority")],
            this.program.programId
        );
        const [reserve] = await anchor.web3.PublicKey.findProgramAddress(
            [trancheConfig.publicKey.toBuffer(), reserveMint.toBuffer()],
            this.program.programId
        );

        await this.program.methods
            .initialize(initData)
            .accounts({
                payer: this.provider.wallet.publicKey,
                owner: owner ?? this.provider.wallet.publicKey,
                trancheConfig: trancheConfig.publicKey,
                trancheAuthority,
                rateProgram: ratePlugin.getProgramId(),
                rateProgramState: ratePlugin.rateStateId,
                redeemLogicProgram: redeemLogicPlugin.getProgramId(),
                redeemLogicProgramState: redeemLogicPlugin.redeemLogicStateId,
                reserveMint,
                reserve,
                juniorTrancheMint: juniorTrancheMint.publicKey,
                seniorTrancheMint: seniorTrancheMint.publicKey,
            })
            .signers([juniorTrancheMint, seniorTrancheMint, trancheConfig])
            .rpc();

        this.seniorTrancheMint = seniorTrancheMint.publicKey;
        this.juniorTrancheMint = juniorTrancheMint.publicKey;
        this.trancheId = trancheConfig.publicKey;
        this.trancheAuthority = trancheAuthority;
        this.reserveMint = reserveMint;
        this.reserve = reserve;
        this.ratePlugin = ratePlugin;
        this.redeemLogicPlugin = redeemLogicPlugin;
        
    }

    async getDepositIx(
        seniorDepositAmount: number,
        juniorDepositAmount: number,
        userReserveToken: PublicKey,
        userSeniorTrancheTokenAccount: PublicKey,
        userJuniorTrancheTokenAccount: PublicKey
    ): Promise<anchor.web3.TransactionInstruction>{
        return await this.program.methods
        .deposit({
            reserveQuantity: [new anchor.BN(seniorDepositAmount), new anchor.BN(juniorDepositAmount)],
        })
        .accounts({
            signer: this.provider.wallet.publicKey,
            trancheConfig: this.trancheId,
            trancheAuthority: this.trancheAuthority,
            reserve: this.reserve,
            userReserveToken,
            seniorTrancheMint: this.seniorTrancheMint,
            juniorTrancheMint: this.juniorTrancheMint,
            seniorTrancheDest: userSeniorTrancheTokenAccount,
            juniorTrancheDest: userJuniorTrancheTokenAccount,
        })
        .instruction();
    }

    async getRedeemIx(
        seniorDepositAmount: number,
        juniorDepositAmount: number,
        userReserveToken: anchor.web3.PublicKey,
        seniorTrancheTokenAccount: anchor.web3.PublicKey,
        juniorTrancheTokenAccount: anchor.web3.PublicKey
    ): Promise<anchor.web3.Transaction> {
       
        const redeemIx = new anchor.web3.Transaction();
        redeemIx.add(await this.ratePlugin.getRefreshIX());
        redeemIx.add(await this.getRefreshTrancheFairValueIX());
        redeemIx.add(
            await this.program.methods
            .redeem({
                trancheQuantity: [new anchor.BN(seniorDepositAmount), new anchor.BN(juniorDepositAmount)],
            })
            .accounts({
                signer: this.provider.wallet.publicKey,
                trancheConfig: this.trancheId,
                trancheAuthority: this.trancheAuthority,
                reserve: this.reserve,
                userReserveToken,
                seniorTrancheMint: this.seniorTrancheMint,
                juniorTrancheMint: this.juniorTrancheMint,
                seniorTrancheSource: seniorTrancheTokenAccount,
                juniorTrancheSource: juniorTrancheTokenAccount,
            })
            .instruction()
        );
       
        return redeemIx;
    }
    async getCollectFee(): Promise<anchor.web3.TransactionInstruction>{
        return await this.program.methods
        .collectFee()
        .accounts({
            trancheConfig: this.trancheId,
            trancheAuthority: this.trancheAuthority,
            reserve: this.reserve,
            destReserve: this.reserve,
            owner: this.provider.wallet.publicKey,
            tokenProgram: this.program.programId
        }).instruction();
    }

}

