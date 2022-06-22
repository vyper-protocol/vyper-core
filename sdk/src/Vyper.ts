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
import { RedeemLogicLending } from "../../target/types/redeem_logic_lending";
import idlRedeemLogicLending from "../../target/idl/redeem_logic_lending.json";
import { RedeemLogicState } from "./RedeemLogicState";

export class Vyper {

    program: anchor.Program<VyperCore>;
    provider: anchor.AnchorProvider;
    trancheId: PublicKey;
    redeemLendingprogram: anchor.Program<RedeemLogicLending>;
    redeemLendingStateId: PublicKey;

    static create(provider: anchor.AnchorProvider, vyperCoreId: PublicKey): Vyper {
        const client = new Vyper();
        const program = new anchor.Program(idlVyperCore as any, vyperCoreId, provider) as anchor.Program<VyperCore>;
        client.program = program;
        client.provider = provider;
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

    createRedeemLendingProgram(provider: anchor.AnchorProvider, redeemLendingId: PublicKey) {
        const program = new anchor.Program(idlRedeemLogicLending as any, redeemLendingId, provider) as anchor.Program<RedeemLogicLending>;
        this.redeemLendingprogram = program;
    }

    async getRedeemLendingConfiguration(redeemLendingStateId?: PublicKey) {

        if (!redeemLendingStateId) {
            redeemLendingStateId = this.redeemLendingStateId;
        }
        const redeemLendingState = await this.redeemLendingprogram.account.redeemLogicConfig.fetch(redeemLendingStateId);
        const redeemLogicState = new RedeemLogicState(
            redeemLendingState.interestSplit,
            redeemLendingState.fixedFeePerTranche.toNumber(),
            redeemLendingState.owner
        )
        return redeemLogicState;
    }


}

