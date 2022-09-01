use super::client_state::NearClientState;
use super::consensus_state::NearConsensusState;
use super::error::Error as NearError;
use super::header::NearHeader;
use crate::clients::host_functions::{HostFunctionsManager, HostFunctionsProvider};
use crate::core::ics02_client::client_consensus::AnyConsensusState;
use crate::core::ics02_client::client_def::{ClientDef, ConsensusUpdateResult};
use crate::core::ics02_client::client_state::AnyClientState;
use crate::core::ics02_client::error::Error;
use crate::core::ics03_connection::connection::ConnectionEnd;
use crate::core::ics04_channel::channel::ChannelEnd;
use crate::core::ics04_channel::commitment::{AcknowledgementCommitment, PacketCommitment};
use crate::core::ics04_channel::packet::Sequence;
use crate::core::ics23_commitment::commitment::{
    CommitmentPrefix, CommitmentProofBytes, CommitmentRoot,
};
use crate::core::ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId};
use crate::core::ics26_routing::context::ReaderContext;
use crate::prelude::*;
use crate::Height;
use core::marker::PhantomData;
use near_lite_client::validate_light_block;

#[derive(Debug, Clone)]
pub struct NearClient<T>(PhantomData<T>);

impl<HostFunctions> Default for NearClient<HostFunctions> {
    fn default() -> Self {
        Self(PhantomData::default())
    }
}

impl<T: HostFunctionsProvider> ClientDef for NearClient<T> {
    /// The data that we need to update the [`ClientState`] to a new block height
    type Header = NearHeader;

    /// The data that we need to know, to validate incoming headers and update the state
    /// of our [`ClientState`]. Ususally this will store:
    ///    - The current epoch
    ///    - The current validator set
    ///
    /// ```rust,no_run
    /// pub struct NearLightClientState {
    ///     head: LightClientBlockView,
    ///     eopch_block_producers: NearBlockProducers,
    ///     next_validators:  Vec<ValidatorStakeView>,
    /// }
    /// ```
    type ClientState = NearClientState;

    /// This is usually just two things, that should be derived from the header:
    ///    - The ibc commitment root hash as described by ics23 (possibly from tx outcome/ state proof)
    ///    - The timestamp of the header.
    type ConsensusState = NearConsensusState;

    // rehydrate client from its own storage, then call this function
    fn verify_header(
        &self,
        _ctx: &dyn ReaderContext,
        _client_id: ClientId,
        mut client_state: Self::ClientState,
        header: Self::Header,
    ) -> Result<(), Error> {
        // TODO: optimize the verification, according to https://nomicon.io/ChainSpec/LightClient
        for header in header.inner {
            validate_light_block::<HostFunctionsManager<T>>(
                &client_state.head,
                &header,
                &client_state.epoch_block_producers,
            )
            .map_err(|e| Error::header_verification_failure(e.to_string()))?;
            if let Some(next_bps) = &header.next_bps {
                client_state
                    .epoch_block_producers
                    .insert(header.inner_lite.next_epoch_id, next_bps.clone());
            }
            client_state.head = header;
        }
        Ok(())
    }

    fn update_state(
        &self,
        ctx: &dyn ReaderContext,
        client_id: ClientId,
        mut client_state: Self::ClientState,
        header: Self::Header,
    ) -> Result<(Self::ClientState, ConsensusUpdateResult), Error> {
        // 1. create new client state from this header, return that.
        // 2. as well as all the necessary consensus states.
        //
        // []--[]--[]--[]--[]--[]--[]--[]--[]--[]
        // []--[]--[]--[]--[]--[]--[]--[]--[]--[]
        // []--[]--[]--[]--[]--[]--[]--[]--[]--[]
        // []--[]--[]--[]--[]--[]--[]--[]--[]--[]
        // 11  12  13  14  15  16  17  18  19  20 <- block merkle root
        // ^                                    ^
        // |    <-------consensus states----->  |
        // current state                       new state (for the previous block)

        let mut cs_states = vec![];
        for header in header.inner {
            if let Some(next_bps) = &header.next_bps {
                client_state
                    .epoch_block_producers
                    .insert(header.inner_lite.next_epoch_id, next_bps.clone());
            }
            if client_state.prev_lite_block.next_bps.is_some() {
                client_state.epoch_num += 1;
            }

            let height = Height::new(
                client_state.prev_epoch_num,
                client_state.prev_lite_block.inner_lite.height,
            );

            if !ctx.consensus_state(&client_id, height).is_ok() {
                // the state root, needed for the consensus state
                // is only available for the previous block (not for the head)
                cs_states.push((
                    height,
                    AnyConsensusState::Near(NearConsensusState::from_header(
                        &client_state.prev_lite_block,
                        header.inner_lite.prev_state_root,
                    )?),
                ));
            }
            client_state.prev_epoch_num = client_state.epoch_num;
            client_state.prev_lite_block = client_state.head;
            client_state.head = header;
        }

        Ok((client_state, ConsensusUpdateResult::Batch(cs_states)))
    }

    fn update_state_on_misbehaviour(
        &self,
        _client_state: Self::ClientState,
        _header: Self::Header,
    ) -> Result<Self::ClientState, Error> {
        todo!()
    }

    fn check_for_misbehaviour(
        &self,
        _ctx: &dyn ReaderContext,
        _client_id: ClientId,
        _client_state: Self::ClientState,
        _header: Self::Header,
    ) -> Result<bool, Error> {
        Ok(false)
    }

    fn verify_upgrade_and_update_state(
        &self,
        _client_state: &Self::ClientState,
        _consensus_state: &Self::ConsensusState,
        _proof_upgrade_client: Vec<u8>,
        _proof_upgrade_consensus_state: Vec<u8>,
    ) -> Result<(Self::ClientState, ConsensusUpdateResult), Error> {
        todo!()
    }

    fn verify_client_consensus_state(
        &self,
        _ctx: &dyn ReaderContext,
        _client_state: &Self::ClientState,
        _height: Height,
        _prefix: &CommitmentPrefix,
        _proof: &CommitmentProofBytes,
        _root: &CommitmentRoot,
        _client_id: &ClientId,
        _consensus_height: Height,
        _expected_consensus_state: &AnyConsensusState,
    ) -> Result<(), Error> {
        todo!()
    }

    // Consensus state will be verified in the verification functions  before these are called
    fn verify_connection_state(
        &self,
        _ctx: &dyn ReaderContext,
        _client_id: &ClientId,
        _client_state: &Self::ClientState,
        _height: Height,
        _prefix: &CommitmentPrefix,
        _proof: &CommitmentProofBytes,
        _root: &CommitmentRoot,
        _connection_id: &ConnectionId,
        _expected_connection_end: &ConnectionEnd,
    ) -> Result<(), Error> {
        todo!()
    }

    fn verify_channel_state(
        &self,
        _ctx: &dyn ReaderContext,
        _client_id: &ClientId,
        _client_state: &Self::ClientState,
        _height: Height,
        _prefix: &CommitmentPrefix,
        _proof: &CommitmentProofBytes,
        _root: &CommitmentRoot,
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _expected_channel_end: &ChannelEnd,
    ) -> Result<(), Error> {
        todo!()
    }

    fn verify_client_full_state(
        &self,
        _ctx: &dyn ReaderContext,
        _client_state: &Self::ClientState,
        _height: Height,
        _prefix: &CommitmentPrefix,
        _proof: &CommitmentProofBytes,
        _root: &CommitmentRoot,
        _client_id: &ClientId,
        _expected_client_state: &AnyClientState,
    ) -> Result<(), Error> {
        todo!()
    }

    fn verify_packet_data(
        &self,
        _ctx: &dyn ReaderContext,
        _client_id: &ClientId,
        _client_state: &Self::ClientState,
        _height: Height,
        _connection_end: &ConnectionEnd,
        _proof: &CommitmentProofBytes,
        _root: &CommitmentRoot,
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _sequence: Sequence,
        _commitment: PacketCommitment,
    ) -> Result<(), Error> {
        todo!()
    }

    fn verify_packet_acknowledgement(
        &self,
        _ctx: &dyn ReaderContext,
        _client_id: &ClientId,
        _client_state: &Self::ClientState,
        _height: Height,
        _connection_end: &ConnectionEnd,
        _proof: &CommitmentProofBytes,
        _root: &CommitmentRoot,
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _sequence: Sequence,
        _ack: AcknowledgementCommitment,
    ) -> Result<(), Error> {
        todo!()
    }

    fn verify_next_sequence_recv(
        &self,
        _ctx: &dyn ReaderContext,
        _client_id: &ClientId,
        _client_state: &Self::ClientState,
        _height: Height,
        _connection_end: &ConnectionEnd,
        _proof: &CommitmentProofBytes,
        _root: &CommitmentRoot,
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _sequence: Sequence,
    ) -> Result<(), Error> {
        todo!()
    }

    fn verify_packet_receipt_absence(
        &self,
        _ctx: &dyn ReaderContext,
        _client_id: &ClientId,
        _client_state: &Self::ClientState,
        _height: Height,
        _connection_end: &ConnectionEnd,
        _proof: &CommitmentProofBytes,
        _root: &CommitmentRoot,
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _sequence: Sequence,
    ) -> Result<(), Error> {
        todo!()
    }
}
