//! ICS 07: Tendermint Client implements a client verification algorithm for blockchains which use
//! the Tendermint consensus algorithm.

pub mod client_def;
pub mod client_state;
pub mod consensus_state;
pub mod error;
pub mod header;
pub mod misbehaviour;
#[cfg(any(test, feature = "mocks"))]
pub mod mock;

#[cfg(test)]
mod tests {
    use crate::ics07_tendermint::client_state::test_util::get_dummy_tendermint_client_state;
    use crate::ics07_tendermint::client_state::ClientState as TendermintClientState;
    use crate::ics07_tendermint::header::test_util::{
        get_dummy_ics07_header, get_dummy_tendermint_header,
    };
    use crate::ics07_tendermint::mock::context::with_client_parametrized;
    use crate::ics07_tendermint::mock::{
        host::MockHostType, AnyClientState, AnyConsensusState, AnyHeader, MockClientTypes,
    };
    use ibc::core::ics02_client::client_state::ClientState;
    use ibc::core::ics02_client::client_type::ClientType;
    use ibc::core::ics02_client::context::ClientReader;
    use ibc::core::ics02_client::handler::{dispatch, ClientResult};
    use ibc::core::ics02_client::header::Header;
    use ibc::core::ics02_client::msgs::update_client::MsgUpdateAnyClient;
    use ibc::core::ics02_client::msgs::{create_client::MsgCreateAnyClient, ClientMsg};
    use ibc::core::ics02_client::trust_threshold::TrustThreshold;
    use ibc::core::ics23_commitment::specs::ProofSpecs;
    use ibc::core::ics24_host::identifier::{ChainId, ClientId};
    use ibc::core::ics26_routing::msgs::Ics26Envelope;
    use ibc::events::IbcEvent;
    use ibc::handler::HandlerOutput;
    use ibc::mock::context::MockContext;
    use ibc::prelude::*;
    use ibc::relayer::ics18_relayer::context::Ics18Context;
    use ibc::relayer::ics18_relayer::utils::build_client_update_datagram;
    use ibc::test_utils::get_dummy_account_id;
    use ibc::Height;
    use ibc_proto::ibc::core::client::v1::{MsgCreateClient, MsgUpdateClient};
    use std::time::Duration;
    use test_log::test;
    use tracing::debug;

    #[test]
    fn msg_create_client_serialization() {
        let signer = get_dummy_account_id();

        let tm_header = get_dummy_tendermint_header();
        let tm_client_state = get_dummy_tendermint_client_state(tm_header.clone());

        let msg = MsgCreateAnyClient::<MockContext<MockClientTypes>>::new(
            tm_client_state,
            AnyConsensusState::Tendermint(tm_header.try_into().unwrap()),
            signer,
        )
        .unwrap();

        let raw = MsgCreateClient::from(msg.clone());
        let msg_back =
            MsgCreateAnyClient::<MockContext<MockClientTypes>>::try_from(raw.clone()).unwrap();
        let raw_back = MsgCreateClient::from(msg_back.clone());
        assert_eq!(msg, msg_back);
        assert_eq!(raw, raw_back);
    }

    #[test]
    fn test_tm_create_client_ok() {
        let signer = get_dummy_account_id();

        let ctx = MockContext::default();

        let tm_header = get_dummy_tendermint_header();

        let tm_client_state = AnyClientState::Tendermint(
            TendermintClientState::new(
                tm_header.chain_id.clone().into(),
                TrustThreshold::ONE_THIRD,
                Duration::from_secs(64000),
                Duration::from_secs(128000),
                Duration::from_millis(3000),
                Height::new(0, u64::from(tm_header.height)),
                ProofSpecs::default(),
                vec!["".to_string()],
            )
            .unwrap(),
        );

        let msg = MsgCreateAnyClient::<MockContext<MockClientTypes>>::new(
            tm_client_state,
            AnyConsensusState::Tendermint(tm_header.try_into().unwrap()),
            signer,
        )
        .unwrap();

        let output = dispatch(&ctx, ClientMsg::CreateClient(msg.clone()));

        match output {
            Ok(HandlerOutput {
                result, mut events, ..
            }) => {
                assert_eq!(events.len(), 1);
                let event = events.pop().unwrap();
                let expected_client_id = ClientId::new(ClientType::Tendermint, 0).unwrap();
                assert!(
                    matches!(event, IbcEvent::CreateClient(ref e) if e.client_id() == &expected_client_id)
                );
                assert_eq!(event.height(), ctx.host_height());
                match result {
                    ClientResult::Create(create_res) => {
                        assert_eq!(create_res.client_type, ClientType::Tendermint);
                        assert_eq!(create_res.client_id, expected_client_id);
                        assert_eq!(create_res.client_state, msg.client_state);
                        assert_eq!(create_res.consensus_state, msg.consensus_state);
                    }
                    _ => {
                        panic!("expected result of type ClientResult::CreateResult");
                    }
                }
            }
            Err(err) => {
                panic!("unexpected error: {}", err);
            }
        }
    }

    #[test]
    fn msg_update_client_serialization() {
        let client_id: ClientId = "tendermint".parse().unwrap();
        let signer = get_dummy_account_id();

        let header = get_dummy_ics07_header();

        let msg = MsgUpdateAnyClient::<MockContext<MockClientTypes>>::new(
            client_id,
            AnyHeader::Tendermint(header),
            signer,
        );
        let raw = MsgUpdateClient::from(msg.clone());
        let msg_back = MsgUpdateAnyClient::try_from(raw.clone()).unwrap();
        let raw_back = MsgUpdateClient::from(msg_back.clone());
        assert_eq!(msg, msg_back);
        assert_eq!(raw, raw_back);
    }

    #[test]
    /// Serves to test both ICS 26 `dispatch` & `build_client_update_datagram` functions.
    /// Implements a "ping pong" of client update messages, so that two chains repeatedly
    /// process a client update message and update their height in succession.
    fn client_update_ping_pong() {
        let chain_a_start_height = Height::new(1, 11);
        let chain_b_start_height = Height::new(1, 20);
        let client_on_b_for_a_height = Height::new(1, 10); // Should be smaller than `chain_a_start_height`
        let client_on_a_for_b_height = Height::new(1, 20); // Should be smaller than `chain_b_start_height`
        let num_iterations = 4;

        let client_on_a_for_b = ClientId::new(ClientType::Tendermint, 0).unwrap();
        let client_on_b_for_a = ClientId::new(ClientType::Mock, 0).unwrap();

        // Create two mock contexts, one for each chain.
        let ctx_a = MockContext::<MockClientTypes>::new(
            ChainId::new("mockgaiaA".to_string(), 1),
            MockHostType::Mock,
            5,
            chain_a_start_height,
        );
        let mut ctx_a = with_client_parametrized(
            ctx_a,
            &client_on_a_for_b,
            client_on_a_for_b_height,
            Some(ClientType::Tendermint), // The target host chain (B) is synthetic TM.
            Some(client_on_a_for_b_height),
        );
        let ctx_b = MockContext::<MockClientTypes>::new(
            ChainId::new("mockgaiaB".to_string(), 1),
            MockHostType::SyntheticTendermint,
            5,
            chain_b_start_height,
        );
        let mut ctx_b = with_client_parametrized(
            ctx_b,
            &client_on_b_for_a,
            client_on_b_for_a_height,
            Some(ClientType::Mock), // The target host chain is mock.
            Some(client_on_b_for_a_height),
        );

        for _i in 0..num_iterations {
            // Update client on chain B to latest height of A.
            // - create the client update message with the latest header from A
            let a_latest_header = ctx_a.query_latest_header().unwrap();
            assert_eq!(
                a_latest_header.client_type(),
                ClientType::Mock,
                "Client type verification in header failed for context A (Mock); got {:?} but expected {:?}",
                a_latest_header.client_type(),
                ClientType::Mock
            );

            let client_msg_b_res =
                build_client_update_datagram(&ctx_b, &client_on_b_for_a, a_latest_header);

            assert!(
                client_msg_b_res.is_ok(),
                "create_client_update failed for context destination {:?}, error: {:?}",
                ctx_b,
                client_msg_b_res
            );

            let client_msg_b = client_msg_b_res.unwrap();

            // - send the message to B. We bypass ICS18 interface and call directly into
            // MockContext `recv` method (to avoid additional serialization steps).
            let dispatch_res_b = ctx_b.deliver(Ics26Envelope::Ics2Msg(client_msg_b));
            let validation_res = ctx_b.validate();
            assert!(
                validation_res.is_ok(),
                "context validation failed with error {:?} for context {:?}",
                validation_res,
                ctx_b
            );

            // Check if the update succeeded.
            assert!(
                dispatch_res_b.is_ok(),
                "Dispatch failed for host chain b with error: {:?}",
                dispatch_res_b
            );
            let client_height_b = ctx_b
                .query_client_full_state(&client_on_b_for_a)
                .unwrap()
                .latest_height();
            assert_eq!(client_height_b, ctx_a.query_latest_height());

            // Update client on chain B to latest height of B.
            // - create the client update message with the latest header from B
            // The test uses LightClientBlock that does not store the trusted height
            let b_latest_header = match ctx_b.query_latest_header().unwrap() {
                AnyHeader::Tendermint(header) => {
                    let th = header.height();
                    let mut hheader = header.clone();
                    hheader.trusted_height = th.decrement().unwrap();
                    AnyHeader::Tendermint(hheader)
                }
                other => other,
            };

            assert_eq!(
                b_latest_header.client_type(),
                ClientType::Tendermint,
                "Client type verification in header failed for context B (TM); got {:?} but expected {:?}",
                b_latest_header.client_type(),
                ClientType::Tendermint
            );

            let client_msg_a_res =
                build_client_update_datagram(&ctx_a, &client_on_a_for_b, b_latest_header);

            assert!(
                client_msg_a_res.is_ok(),
                "create_client_update failed for context destination {:?}, error: {:?}",
                ctx_a,
                client_msg_a_res
            );

            let client_msg_a = client_msg_a_res.unwrap();

            debug!("client_msg_a = {:?}", client_msg_a);

            // - send the message to A
            let dispatch_res_a = ctx_a.deliver(Ics26Envelope::Ics2Msg(client_msg_a));
            let validation_res = ctx_a.validate();
            assert!(
                validation_res.is_ok(),
                "context validation failed with error {:?} for context {:?}",
                validation_res,
                ctx_a
            );

            // Check if the update succeeded.
            assert!(
                dispatch_res_a.is_ok(),
                "Dispatch failed for host chain a with error: {:?}",
                dispatch_res_a
            );
            let client_height_a = ctx_a
                .query_client_full_state(&client_on_a_for_b)
                .unwrap()
                .latest_height();
            assert_eq!(client_height_a, ctx_b.query_latest_height());
        }
    }
}
