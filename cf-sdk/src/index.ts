import { PublicKey } from "@solana/web3.js";

export * from "./client";
export * from "./types";
export * from "./adapters";

export { CastleLendingAggregator } from "./castle_lending_aggregator";

export const PROGRAM_ID = new PublicKey(
  "6hSKFKsZvksTb4M7828LqWsquWnyatoRwgZbcpeyfWRb"
  //"E5xEvrNhrknmgGbRv8iDDqHsgqG1xeMEdfPjL8i4YEVo"
);
