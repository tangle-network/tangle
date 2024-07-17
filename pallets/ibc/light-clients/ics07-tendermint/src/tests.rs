// Copyright 2022 ComposableFi
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#[test]
fn test_update_synthetic_tendermint_client_lower_height() {
    let client_id = ClientId::new(ClientState::<()>::client_type(), 0).unwrap();
    let client_height = Height::new(1, 20);

    let client_update_height = Height::new(1, 19);

    let chain_start_height = Height::new(1, 11);

    let ctx = MockContext::new(
        ChainId::new("mockgaiaA".to_string(), 1),
        HostType::Mock,
        5,
        chain_start_height,
    )
    .with_client_parametrized(
        &client_id,
        client_height,
        Some(ClientState::<()>::client_type()), // The target host chain (B) is synthetic TM.
        Some(client_height),
    );

    let ctx_b = MockContext::new(
        ChainId::new("mockgaiaB".to_string(), 1),
        HostType::SyntheticTendermint,
        5,
        client_height,
    );

    let signer = get_dummy_account_id();

    let block_ref = ctx_b.host_block(client_update_height);
    let latest_header: AnyHeader = block_ref.cloned().map(Into::into).unwrap();

    let msg = MsgUpdateAnyClient {
        client_id,
        header: latest_header,
        signer,
    };

    let output = dispatch(&ctx, ClientMsg::UpdateClient(msg));

    match output {
        Ok(_) => {
            panic!("update handler result has incorrect type");
        }
        Err(err) => match err.detail() {
            ErrorDetail::HeaderVerificationFailure(_) => {}
            _ => panic!("unexpected error: {:?}", err),
        },
    }
}

#[test]
fn test_history_manipulation() {
    pub struct Test {
        name: String,
        ctx: MockContext,
    }
    let cv = 1; // The version to use for all chains.

    let tests: Vec<Test> = vec![
        Test {
            name: "Empty history, small pruning window".to_string(),
            ctx: MockContext::new(
                ChainId::new("mockgaia".to_string(), cv),
                HostType::Mock,
                2,
                Height::new(cv, 1),
            ),
        },
        Test {
            name: "[Synthetic TM host] Empty history, small pruning window".to_string(),
            ctx: MockContext::new(
                ChainId::new("mocksgaia".to_string(), cv),
                HostType::SyntheticTendermint,
                2,
                Height::new(cv, 1),
            ),
        },
        Test {
            name: "Large pruning window".to_string(),
            ctx: MockContext::new(
                ChainId::new("mockgaia".to_string(), cv),
                HostType::Mock,
                30,
                Height::new(cv, 2),
            ),
        },
        Test {
            name: "[Synthetic TM host] Large pruning window".to_string(),
            ctx: MockContext::new(
                ChainId::new("mocksgaia".to_string(), cv),
                HostType::SyntheticTendermint,
                30,
                Height::new(cv, 2),
            ),
        },
        Test {
            name: "Small pruning window".to_string(),
            ctx: MockContext::new(
                ChainId::new("mockgaia".to_string(), cv),
                HostType::Mock,
                3,
                Height::new(cv, 30),
            ),
        },
        Test {
            name: "[Synthetic TM host] Small pruning window".to_string(),
            ctx: MockContext::new(
                ChainId::new("mockgaia".to_string(), cv),
                HostType::SyntheticTendermint,
                3,
                Height::new(cv, 30),
            ),
        },
        Test {
            name: "Small pruning window, small starting height".to_string(),
            ctx: MockContext::new(
                ChainId::new("mockgaia".to_string(), cv),
                HostType::Mock,
                3,
                Height::new(cv, 2),
            ),
        },
        Test {
            name: "[Synthetic TM host] Small pruning window, small starting height".to_string(),
            ctx: MockContext::new(
                ChainId::new("mockgaia".to_string(), cv),
                HostType::SyntheticTendermint,
                3,
                Height::new(cv, 2),
            ),
        },
        Test {
            name: "Large pruning window, large starting height".to_string(),
            ctx: MockContext::new(
                ChainId::new("mockgaia".to_string(), cv),
                HostType::Mock,
                50,
                Height::new(cv, 2000),
            ),
        },
        Test {
            name: "[Synthetic TM host] Large pruning window, large starting height".to_string(),
            ctx: MockContext::new(
                ChainId::new("mockgaia".to_string(), cv),
                HostType::SyntheticTendermint,
                50,
                Height::new(cv, 2000),
            ),
        },
    ];

    for mut test in tests {
        // All tests should yield a valid context after initialization.
        assert!(
            test.ctx.validate().is_ok(),
            "failed in test {} while validating context {:?}",
            test.name,
            test.ctx
        );

        let current_height = test.ctx.latest_height();

        // After advancing the chain's height, the context should still be valid.
        test.ctx.advance_host_chain_height();
        assert!(
            test.ctx.validate().is_ok(),
            "failed in test {} while validating context {:?}",
            test.name,
            test.ctx
        );

        let next_height = current_height.increment();
        assert_eq!(
            test.ctx.latest_height(),
            next_height,
            "failed while increasing height for context {:?}",
            test.ctx
        );
        if current_height > Height::new(cv, 0) {
            assert_eq!(
                test.ctx.host_block(current_height).unwrap().height(),
                current_height,
                "failed while fetching height {:?} of context {:?}",
                current_height,
                test.ctx
            );
        }
    }
}
