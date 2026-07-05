pub struct NetworkInfo {
    pub contract_address: &'static str,
    pub deployment_block: u64,
    pub deployment_block_hash: &'static str,
}

pub const TESTNET: NetworkInfo = NetworkInfo {
    contract_address: "0xe916A48dE922E8964542F4c4c66Ec4837bBE3445",
    deployment_block: 7797344,
    deployment_block_hash: "0x63ad5c63d3eb642f3d4a58ede0986fec63cda4b1b0c3ecebf32260eb320fe4fc",
};

pub fn resolve(name: &str) -> Option<&'static NetworkInfo> {
    match name {
        "testnet" => Some(&TESTNET),
        _ => None,
    }
}
