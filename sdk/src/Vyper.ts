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
import idlRateMock from "../../target/idl/rate_mock.json";
import { RateMock } from "../../target/types/rate_mock";
import { RateState } from "./RateMockState";

export class Vyper {

    program: anchor.Program<VyperCore>;
    provider: anchor.AnchorProvider;
    trancheId: PublicKey;
    redeemLendingProgram: anchor.Program<RedeemLogicLending>;
    redeemLendingStateId: PublicKey;
    rateMockProgram: anchor.Program<RateMock>;
    rateMockStateId: PublicKey;

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
        this.redeemLendingProgram = program;
    }

    async getRedeemLendingConfiguration(redeemLendingStateId?: PublicKey) {

        if (!redeemLendingStateId) {
            redeemLendingStateId = this.redeemLendingStateId;
        }
        const redeemLendingState = await this.redeemLendingProgram.account.redeemLogicConfig.fetch(redeemLendingStateId);
        const redeemLogicState = new RedeemLogicState(
            redeemLendingState.interestSplit,
            redeemLendingState.fixedFeePerTranche.toNumber(),
            redeemLendingState.owner
        )
        return redeemLogicState;
    }

    createRateMockProgram(provider: anchor.AnchorProvider, rateMockId: PublicKey) {
        const program = new anchor.Program(idlRateMock as any, rateMockId, provider) as anchor.Program<RateMock>;
        this.rateMockProgram = program;
    }

    async getRateMockState(rateMockStateId?: PublicKey) {


        if (!rateMockStateId) {
            rateMockStateId = this.rateMockStateId;
        }
        const rateMockState = await this.rateMockProgram.account.rateState.fetch(rateMockStateId);
        const rateState = new RateState(
            rateMockState.fairValue,
            rateMockState.refreshedSlot.toNumber(),
        )
        return rateState;
    }


}

