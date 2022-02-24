// Collator flags
var flags = ["--force-authoring", "--", "--execution=wasm"];

var config = {
	relaychain: {
        // change bin path accordingly
		bin: "../../egg-testnet/polkadot/target/release/polkadot",
		chain: "rococo-local",
		nodes: [
			{
				name: "alice",
				wsPort: 9944,
                rpcPort: 5555,
				port: 30444,
			},
			{
				name: "bob",
				wsPort: 9955,
                rpcPort: 6666,
				port: 30555,
			},
			{
				name: "charlie",
				wsPort: 9966,
                rpcPort: 7777,
				port: 30666,
			},
			{
				name: "dave",
				wsPort: 9977,
                rpcPort: 8888,
				port: 30777,
			},
		],
		genesis: {
			runtime: {
				runtime_genesis_config: {
					configuration: {
						config: {
							validation_upgrade_frequency: 10,
							validation_upgrade_delay: 10,
						},
					},
				},
			},
		},
	},
	parachains: [
		{
            // change bin path accordingly
			bin: "../target/release/egg-collator",
			id: "2000",
			balance: "1000000000000000000000",
			nodes: [
				{
					wsPort: 9988,
					port: 31200,
                    rpcPort: 4444,
					name: "alice",
					flags,
				},
			],
		},
		{
			bin: "../target/release/egg-collator",
			id: "2001",
			balance: "1000000000000000000000",
			nodes: [
				{
					wsPort: 9999,
					port: 31300,
                    rpcPort: 3333,
					name: "charlie",
					flags,
				},
			],
		},
        {
			bin: "../target/release/egg-collator",
			id: "2002",
			balance: "1000000000000000000000",
			nodes: [
				{
					wsPort: 9999,
					port: 31400,
                    rpcPort: 2222,
					name: "bob",
					flags,
				},
			],
		},
	],
    // TODO: add DKG specific types
	types: {},
	finalization: false,
};

module.exports = config;