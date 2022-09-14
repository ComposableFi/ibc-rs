use crate::any::client_state::AnyClientState;
use crate::any::consensus_state::AnyConsensusState;
use crate::any::mock::MockClientTypes;
use crate::ics07_tendermint::client_state::ClientState as TMClientState;
use crate::ics07_tendermint::mock::host::MockHostBlock;
use crate::ics11_beefy::client_state::{ClientState, RelayChain};
use crate::ics11_beefy::consensus_state::ConsensusState;
use core::time::Duration;
use frame_support::log::debug;
use ibc::core::ics02_client::client_type::ClientType;
use ibc::core::ics24_host::identifier::ChainId;
use ibc::core::ics24_host::identifier::ClientId;
use ibc::mock::client_state::{MockClientRecord, MockClientState, MockConsensusState};
use ibc::mock::context::MockContext;
use ibc::mock::header::MockHeader;
use ibc::prelude::*;
use ibc::timestamp::Timestamp;
use ibc::Height;
use std::ops::Sub;
use tendermint::block::Header;
use tendermint::Time;

/// Similar to `with_client`, this function associates a client record to this context, but
/// additionally permits to parametrize two details of the client. If `client_type` is None,
/// then the client will have type Mock, otherwise the specified type. If
/// `consensus_state_height` is None, then the client will be initialized with a consensus
/// state matching the same height as the client state (`client_state_height`).
pub fn with_client_parametrized(
    ctx: MockContext<MockClientTypes>,
    client_id: &ClientId,
    client_state_height: Height,
    client_type: Option<ClientType>,
    consensus_state_height: Option<Height>,
) -> MockContext<MockClientTypes> {
    let cs_height = consensus_state_height.unwrap_or(client_state_height);

    let client_type = client_type.unwrap_or(ClientType::Mock);
    let (client_state, consensus_state) = match client_type {
        // If it's a mock client, create the corresponding mock states.
        ClientType::Mock => (
            Some(MockClientState::new(MockHeader::new(client_state_height)).into()),
            MockConsensusState::new(MockHeader::new(cs_height)).into(),
        ),
        ClientType::Beefy => (
            Some(get_dummy_beefy_state()),
            get_dummy_beefy_consensus_state(),
        ),
        // If it's a Tendermint client, we need TM states.
        ClientType::Tendermint => {
            let light_block = MockHostBlock::generate_tm_block(
                ctx.host_chain_id.clone(),
                cs_height.revision_height,
                Timestamp::now(),
            );

            let consensus_state = AnyConsensusState::from(light_block.clone());
            let client_state = get_dummy_tendermint_client_state(light_block.signed_header.header);

            // Return the tuple.
            (Some(client_state), consensus_state)
        }
        _ => unimplemented!(),
    };
    let consensus_states = vec![(cs_height, consensus_state)].into_iter().collect();

    debug!("consensus states: {:?}", consensus_states);

    let client_record = MockClientRecord {
        client_type,
        client_state,
        consensus_states,
    };
    ctx.ibc_store
        .lock()
        .unwrap()
        .clients
        .insert(client_id.clone(), client_record);
    ctx
}

pub fn with_client_parametrized_history(
    ctx: MockContext<MockClientTypes>,
    client_id: &ClientId,
    client_state_height: Height,
    client_type: Option<ClientType>,
    consensus_state_height: Option<Height>,
) -> MockContext<MockClientTypes> {
    let cs_height = consensus_state_height.unwrap_or(client_state_height);
    let prev_cs_height = cs_height.clone().sub(1).unwrap_or(client_state_height);

    let client_type = client_type.unwrap_or(ClientType::Mock);
    let now = Timestamp::now();

    let (client_state, consensus_state) = match client_type {
        // If it's a mock client, create the corresponding mock states.
        ClientType::Mock => (
            Some(MockClientState::new(MockHeader::new(client_state_height)).into()),
            MockConsensusState::new(MockHeader::new(cs_height)).into(),
        ),

        ClientType::Beefy => (
            Some(get_dummy_beefy_state()),
            get_dummy_beefy_consensus_state(),
        ),
        // If it's a Tendermint client, we need TM states.
        ClientType::Tendermint => {
            let light_block = MockHostBlock::generate_tm_block(
                ctx.host_chain_id.clone(),
                cs_height.revision_height,
                now,
            );

            let consensus_state = AnyConsensusState::from(light_block.clone());
            let client_state = get_dummy_tendermint_client_state(light_block.signed_header.header);

            // Return the tuple.
            (Some(client_state), consensus_state)
        }
        _ => unimplemented!(),
    };

    let prev_consensus_state = match client_type {
        // If it's a mock client, create the corresponding mock states.
        ClientType::Mock => MockConsensusState::new(MockHeader::new(prev_cs_height)).into(),
        ClientType::Tendermint => {
            let light_block = MockHostBlock::generate_tm_block(
                ctx.host_chain_id.clone(),
                prev_cs_height.revision_height,
                now.sub(ctx.block_time).unwrap(),
            );
            AnyConsensusState::from(light_block)
        }
        _ => unimplemented!(),
    };

    let consensus_states = vec![
        (prev_cs_height, prev_consensus_state),
        (cs_height, consensus_state),
    ]
    .into_iter()
    .collect();

    debug!("consensus states: {:?}", consensus_states);

    let client_record = MockClientRecord {
        client_type,
        client_state,
        consensus_states,
    };

    ctx.ibc_store
        .lock()
        .unwrap()
        .clients
        .insert(client_id.clone(), client_record);
    ctx
}

pub fn get_dummy_beefy_state() -> AnyClientState {
    AnyClientState::Beefy(
        ClientState::new(
            RelayChain::Rococo,
            2000,
            0,
            Default::default(),
            0,
            0,
            Default::default(),
            Default::default(),
        )
        .unwrap(),
    )
}

pub fn get_dummy_beefy_consensus_state() -> AnyConsensusState {
    AnyConsensusState::Beefy(ConsensusState {
        timestamp: Time::now(),
        root: vec![0; 32].into(),
    })
}

pub fn get_dummy_tendermint_client_state(tm_header: Header) -> AnyClientState {
    AnyClientState::Tendermint(
        TMClientState::new(
            ChainId::from(tm_header.chain_id.clone()),
            Default::default(),
            Duration::from_secs(64000),
            Duration::from_secs(128000),
            Duration::from_millis(3000),
            Height::new(
                ChainId::chain_version(tm_header.chain_id.as_str()),
                u64::from(tm_header.height),
            ),
            Default::default(),
            vec!["".to_string()],
        )
        .unwrap(),
    )
}
