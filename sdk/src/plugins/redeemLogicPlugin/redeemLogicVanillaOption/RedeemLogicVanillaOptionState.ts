import { PublicKey } from "@solana/web3.js";

export class RedeemLogicVanillaOptionState {

    isCall: boolean
    isLinear: boolean
    strike: number
    owner: PublicKey

    constructor(isCall: boolean, isLinear: boolean,strike: number, owner: PublicKey) {
        this.isCall = isCall
        this.isLinear = isLinear
        this.strike = strike
        this.owner = owner
    }
} 