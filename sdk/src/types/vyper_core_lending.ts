export type VyperCoreLending = {
    "version": "1.0.0",
    "name": "vyper_core_lending",
    "instructions": [
        {
            "name": "createTranche",
            "accounts": [
                {
                    "name": "authority",
                    "isMut": true,
                    "isSigner": true
                },
                {
                    "name": "trancheConfig",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "mint",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "seniorTrancheMint",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "juniorTrancheMint",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "protocolProgram",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "systemProgram",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "tokenProgram",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "associatedTokenProgram",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "rent",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "clock",
                    "isMut": false,
                    "isSigner": false
                }
            ],
            "args": [
                {
                    "name": "inputData",
                    "type": {
                        "defined": "CreateTrancheConfigInput"
                    }
                },
                {
                    "name": "trancheConfigId",
                    "type": "u64"
                },
                {
                    "name": "trancheConfigBump",
                    "type": "u8"
                },
                {
                    "name": "seniorTrancheMintBump",
                    "type": "u8"
                },
                {
                    "name": "juniorTrancheMintBump",
                    "type": "u8"
                }
            ]
        },
        {
            "name": "updateInterestSplit",
            "accounts": [
                {
                    "name": "authority",
                    "isMut": true,
                    "isSigner": true
                },
                {
                    "name": "trancheConfig",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "systemProgram",
                    "isMut": false,
                    "isSigner": false
                }
            ],
            "args": [
                {
                    "name": "interestSplit",
                    "type": {
                        "array": [
                            "u32",
                            2
                        ]
                    }
                }
            ]
        },
        {
            "name": "updateCapitalSplit",
            "accounts": [
                {
                    "name": "authority",
                    "isMut": true,
                    "isSigner": true
                },
                {
                    "name": "trancheConfig",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "systemProgram",
                    "isMut": false,
                    "isSigner": false
                }
            ],
            "args": [
                {
                    "name": "capitalSplit",
                    "type": {
                        "array": [
                            "u32",
                            2
                        ]
                    }
                }
            ]
        },
        {
            "name": "updateDepositedQuantity",
            "accounts": [
                {
                    "name": "authority",
                    "isMut": true,
                    "isSigner": true
                },
                {
                    "name": "trancheConfig",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "protocolState",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "sourceCollateralAccount",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "systemProgram",
                    "isMut": false,
                    "isSigner": false
                }
            ],
            "args": []
        },
        {
            "name": "createSerumMarket",
            "accounts": [
                {
                    "name": "authority",
                    "isMut": true,
                    "isSigner": true
                },
                {
                    "name": "seniorTrancheMint",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "seniorTrancheSerumVault",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "juniorTrancheMint",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "juniorTrancheSerumVault",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "usdcMint",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "usdcSerumVault",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "market",
                    "isMut": true,
                    "isSigner": true
                },
                {
                    "name": "requestQueue",
                    "isMut": true,
                    "isSigner": true
                },
                {
                    "name": "eventQueue",
                    "isMut": true,
                    "isSigner": true
                },
                {
                    "name": "asks",
                    "isMut": true,
                    "isSigner": true
                },
                {
                    "name": "bids",
                    "isMut": true,
                    "isSigner": true
                },
                {
                    "name": "serumDex",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "systemProgram",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "tokenProgram",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "associatedTokenProgram",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "rent",
                    "isMut": false,
                    "isSigner": false
                }
            ],
            "args": [
                {
                    "name": "vaultSignerNonce",
                    "type": "u8"
                }
            ]
        },
        {
            "name": "deposit",
            "accounts": [
                {
                    "name": "authority",
                    "isMut": true,
                    "isSigner": true
                },
                {
                    "name": "trancheConfig",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "reserveToken",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "sourceLiquidity",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "reserveLiquiditySupply",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "destinationCollateralAccount",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "collateralMint",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "protocolState",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "lendingMarketAccount",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "lendingMarketAuthority",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "pythReserveLiquidityOracle",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "switchboardReserveLiquidityOracle",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "seniorTrancheMint",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "seniorTrancheVault",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "juniorTrancheMint",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "juniorTrancheVault",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "lendingProgram",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "systemProgram",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "tokenProgram",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "associatedTokenProgram",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "rent",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "clock",
                    "isMut": false,
                    "isSigner": false
                }
            ],
            "args": [
                {
                    "name": "quantity",
                    "type": "u64"
                },
                {
                    "name": "mintCount",
                    "type": {
                        "array": [
                            "u64",
                            2
                        ]
                    }
                }
            ]
        },
        {
            "name": "redeem",
            "accounts": [
                {
                    "name": "authority",
                    "isMut": true,
                    "isSigner": true
                },
                {
                    "name": "trancheConfig",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "reserveToken",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "destinationLiquidity",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "reserveLiquiditySupply",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "sourceCollateralAccount",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "collateralMint",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "protocolState",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "lendingMarketAccount",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "lendingMarketAuthority",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "pythReserveLiquidityOracle",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "switchboardReserveLiquidityOracle",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "seniorTrancheMint",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "seniorTrancheVault",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "juniorTrancheMint",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "juniorTrancheVault",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "lendingProgram",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "systemProgram",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "tokenProgram",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "associatedTokenProgram",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "rent",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "clock",
                    "isMut": false,
                    "isSigner": false
                }
            ],
            "args": [
                {
                    "name": "redeemQuantity",
                    "type": {
                        "array": [
                            "u64",
                            2
                        ]
                    }
                }
            ]
        }
    ],
    "accounts": [
        {
            "name": "trancheConfig",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "authority",
                        "type": "publicKey"
                    },
                    {
                        "name": "id",
                        "type": {
                            "array": [
                                "u8",
                                8
                            ]
                        }
                    },
                    {
                        "name": "protocolProgramId",
                        "type": "publicKey"
                    },
                    {
                        "name": "depositedQuantity",
                        "type": {
                            "array": [
                                "u64",
                                2
                            ]
                        }
                    },
                    {
                        "name": "capitalSplit",
                        "type": {
                            "array": [
                                "u32",
                                2
                            ]
                        }
                    },
                    {
                        "name": "interestSplit",
                        "type": {
                            "array": [
                                "u32",
                                2
                            ]
                        }
                    },
                    {
                        "name": "seniorTrancheMint",
                        "type": "publicKey"
                    },
                    {
                        "name": "juniorTrancheMint",
                        "type": "publicKey"
                    },
                    {
                        "name": "createdAt",
                        "type": "u64"
                    },
                    {
                        "name": "createSerum",
                        "type": "bool"
                    },
                    {
                        "name": "trancheConfigBump",
                        "type": "u8"
                    },
                    {
                        "name": "seniorTrancheMintBump",
                        "type": "u8"
                    },
                    {
                        "name": "juniorTrancheMintBump",
                        "type": "u8"
                    }
                ]
            }
        }
    ],
    "types": [
        {
            "name": "CreateTrancheConfigInput",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "capitalSplit",
                        "type": {
                            "array": [
                                "u32",
                                2
                            ]
                        }
                    },
                    {
                        "name": "interestSplit",
                        "type": {
                            "array": [
                                "u32",
                                2
                            ]
                        }
                    },
                    {
                        "name": "createSerum",
                        "type": "bool"
                    }
                ]
            }
        },
        {
            "name": "RedeemTrancheInput",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "quantity",
                        "type": "u64"
                    }
                ]
            }
        },
        {
            "name": "LendingMarketID",
            "type": {
                "kind": "enum",
                "variants": [
                    {
                        "name": "Solend"
                    }
                ]
            }
        },
        {
            "name": "ErrorCode",
            "type": {
                "kind": "enum",
                "variants": [
                    {
                        "name": "GenericError"
                    },
                    {
                        "name": "InvalidInput"
                    },
                    {
                        "name": "InvalidTrancheAmount"
                    },
                    {
                        "name": "InvalidTrancheIdx"
                    },
                    {
                        "name": "InvalidProtocolId"
                    }
                ]
            }
        }
    ]
};

export const IDL: VyperCoreLending = {
    "version": "1.0.0",
    "name": "vyper_core_lending",
    "instructions": [
        {
            "name": "createTranche",
            "accounts": [
                {
                    "name": "authority",
                    "isMut": true,
                    "isSigner": true
                },
                {
                    "name": "trancheConfig",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "mint",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "seniorTrancheMint",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "juniorTrancheMint",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "protocolProgram",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "systemProgram",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "tokenProgram",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "associatedTokenProgram",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "rent",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "clock",
                    "isMut": false,
                    "isSigner": false
                }
            ],
            "args": [
                {
                    "name": "inputData",
                    "type": {
                        "defined": "CreateTrancheConfigInput"
                    }
                },
                {
                    "name": "trancheConfigId",
                    "type": "u64"
                },
                {
                    "name": "trancheConfigBump",
                    "type": "u8"
                },
                {
                    "name": "seniorTrancheMintBump",
                    "type": "u8"
                },
                {
                    "name": "juniorTrancheMintBump",
                    "type": "u8"
                }
            ]
        },
        {
            "name": "updateInterestSplit",
            "accounts": [
                {
                    "name": "authority",
                    "isMut": true,
                    "isSigner": true
                },
                {
                    "name": "trancheConfig",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "systemProgram",
                    "isMut": false,
                    "isSigner": false
                }
            ],
            "args": [
                {
                    "name": "interestSplit",
                    "type": {
                        "array": [
                            "u32",
                            2
                        ]
                    }
                }
            ]
        },
        {
            "name": "updateCapitalSplit",
            "accounts": [
                {
                    "name": "authority",
                    "isMut": true,
                    "isSigner": true
                },
                {
                    "name": "trancheConfig",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "systemProgram",
                    "isMut": false,
                    "isSigner": false
                }
            ],
            "args": [
                {
                    "name": "capitalSplit",
                    "type": {
                        "array": [
                            "u32",
                            2
                        ]
                    }
                }
            ]
        },
        {
            "name": "updateDepositedQuantity",
            "accounts": [
                {
                    "name": "authority",
                    "isMut": true,
                    "isSigner": true
                },
                {
                    "name": "trancheConfig",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "protocolState",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "sourceCollateralAccount",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "systemProgram",
                    "isMut": false,
                    "isSigner": false
                }
            ],
            "args": []
        },
        {
            "name": "createSerumMarket",
            "accounts": [
                {
                    "name": "authority",
                    "isMut": true,
                    "isSigner": true
                },
                {
                    "name": "seniorTrancheMint",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "seniorTrancheSerumVault",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "juniorTrancheMint",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "juniorTrancheSerumVault",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "usdcMint",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "usdcSerumVault",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "market",
                    "isMut": true,
                    "isSigner": true
                },
                {
                    "name": "requestQueue",
                    "isMut": true,
                    "isSigner": true
                },
                {
                    "name": "eventQueue",
                    "isMut": true,
                    "isSigner": true
                },
                {
                    "name": "asks",
                    "isMut": true,
                    "isSigner": true
                },
                {
                    "name": "bids",
                    "isMut": true,
                    "isSigner": true
                },
                {
                    "name": "serumDex",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "systemProgram",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "tokenProgram",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "associatedTokenProgram",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "rent",
                    "isMut": false,
                    "isSigner": false
                }
            ],
            "args": [
                {
                    "name": "vaultSignerNonce",
                    "type": "u8"
                }
            ]
        },
        {
            "name": "deposit",
            "accounts": [
                {
                    "name": "authority",
                    "isMut": true,
                    "isSigner": true
                },
                {
                    "name": "trancheConfig",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "reserveToken",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "sourceLiquidity",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "reserveLiquiditySupply",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "destinationCollateralAccount",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "collateralMint",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "protocolState",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "lendingMarketAccount",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "lendingMarketAuthority",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "pythReserveLiquidityOracle",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "switchboardReserveLiquidityOracle",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "seniorTrancheMint",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "seniorTrancheVault",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "juniorTrancheMint",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "juniorTrancheVault",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "lendingProgram",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "systemProgram",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "tokenProgram",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "associatedTokenProgram",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "rent",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "clock",
                    "isMut": false,
                    "isSigner": false
                }
            ],
            "args": [
                {
                    "name": "quantity",
                    "type": "u64"
                },
                {
                    "name": "mintCount",
                    "type": {
                        "array": [
                            "u64",
                            2
                        ]
                    }
                }
            ]
        },
        {
            "name": "redeem",
            "accounts": [
                {
                    "name": "authority",
                    "isMut": true,
                    "isSigner": true
                },
                {
                    "name": "trancheConfig",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "reserveToken",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "destinationLiquidity",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "reserveLiquiditySupply",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "sourceCollateralAccount",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "collateralMint",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "protocolState",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "lendingMarketAccount",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "lendingMarketAuthority",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "pythReserveLiquidityOracle",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "switchboardReserveLiquidityOracle",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "seniorTrancheMint",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "seniorTrancheVault",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "juniorTrancheMint",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "juniorTrancheVault",
                    "isMut": true,
                    "isSigner": false
                },
                {
                    "name": "lendingProgram",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "systemProgram",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "tokenProgram",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "associatedTokenProgram",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "rent",
                    "isMut": false,
                    "isSigner": false
                },
                {
                    "name": "clock",
                    "isMut": false,
                    "isSigner": false
                }
            ],
            "args": [
                {
                    "name": "redeemQuantity",
                    "type": {
                        "array": [
                            "u64",
                            2
                        ]
                    }
                }
            ]
        }
    ],
    "accounts": [
        {
            "name": "trancheConfig",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "authority",
                        "type": "publicKey"
                    },
                    {
                        "name": "id",
                        "type": {
                            "array": [
                                "u8",
                                8
                            ]
                        }
                    },
                    {
                        "name": "protocolProgramId",
                        "type": "publicKey"
                    },
                    {
                        "name": "depositedQuantity",
                        "type": {
                            "array": [
                                "u64",
                                2
                            ]
                        }
                    },
                    {
                        "name": "capitalSplit",
                        "type": {
                            "array": [
                                "u32",
                                2
                            ]
                        }
                    },
                    {
                        "name": "interestSplit",
                        "type": {
                            "array": [
                                "u32",
                                2
                            ]
                        }
                    },
                    {
                        "name": "seniorTrancheMint",
                        "type": "publicKey"
                    },
                    {
                        "name": "juniorTrancheMint",
                        "type": "publicKey"
                    },
                    {
                        "name": "createdAt",
                        "type": "u64"
                    },
                    {
                        "name": "createSerum",
                        "type": "bool"
                    },
                    {
                        "name": "trancheConfigBump",
                        "type": "u8"
                    },
                    {
                        "name": "seniorTrancheMintBump",
                        "type": "u8"
                    },
                    {
                        "name": "juniorTrancheMintBump",
                        "type": "u8"
                    }
                ]
            }
        }
    ],
    "types": [
        {
            "name": "CreateTrancheConfigInput",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "capitalSplit",
                        "type": {
                            "array": [
                                "u32",
                                2
                            ]
                        }
                    },
                    {
                        "name": "interestSplit",
                        "type": {
                            "array": [
                                "u32",
                                2
                            ]
                        }
                    },
                    {
                        "name": "createSerum",
                        "type": "bool"
                    }
                ]
            }
        },
        {
            "name": "RedeemTrancheInput",
            "type": {
                "kind": "struct",
                "fields": [
                    {
                        "name": "quantity",
                        "type": "u64"
                    }
                ]
            }
        },
        {
            "name": "LendingMarketID",
            "type": {
                "kind": "enum",
                "variants": [
                    {
                        "name": "Solend"
                    }
                ]
            }
        },
        {
            "name": "ErrorCode",
            "type": {
                "kind": "enum",
                "variants": [
                    {
                        "name": "GenericError"
                    },
                    {
                        "name": "InvalidInput"
                    },
                    {
                        "name": "InvalidTrancheAmount"
                    },
                    {
                        "name": "InvalidTrancheIdx"
                    },
                    {
                        "name": "InvalidProtocolId"
                    }
                ]
            }
        }
    ]
};
