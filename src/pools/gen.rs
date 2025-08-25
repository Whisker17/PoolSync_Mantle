use alloy::sol;

// UNISWAP
sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    UniswapV3Factory,
    "src/pools/abis/UniswapV3Factory.json"
);

// MERCHANT MOE
sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    MerchantMoeV2Factory,
    "src/pools/abis/MerchantMoeV2Factory.json"
);

// AGNI
sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    AgniV3Factory,
    "src/pools/abis/AgniV3Factory.json"
);


// ERC20
sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    ERC20,
    "src/pools/abis/ERC20.json"
);

// Data sync contracts

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    V3DataSync,
    "src/abi/V3DataSync.json"
);

sol!(
    #[derive(Debug)]
    #[sol(rpc)]
    V2DataSync,
    "src/abi/V2DataSync.json"
);