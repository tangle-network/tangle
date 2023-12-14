use fp_evm::GenesisAccount;
use sp_core::H160;

pub mod develop;
pub mod mainnet;
pub mod testnet;

pub fn combine_distributions(
	distributions: Vec<Vec<(H160, GenesisAccount)>>,
) -> Vec<(H160, GenesisAccount)> {
	let mut combined = Vec::new();
	for distribution in distributions {
		for (address, account) in distribution {
			combined.push((address, account));
		}
	}
	combined
}
