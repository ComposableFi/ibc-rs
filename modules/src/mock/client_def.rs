use crate::{
	core::{
		ics02_client::{
			client_consensus::AnyConsensusState,
			client_def::{ClientDef, ConsensusUpdateResult},
			client_state::AnyClientState,
			error::Error,
		},
		ics03_connection::connection::ConnectionEnd,
		ics04_channel::{
			channel::ChannelEnd,
			commitment::{AcknowledgementCommitment, PacketCommitment},
			packet::Sequence,
		},
		ics23_commitment::{
			commitment::{CommitmentPrefix, CommitmentProofBytes, CommitmentRoot},
			merkle::apply_prefix,
		},
		ics24_host::{
			identifier::{ChannelId, ClientId, ConnectionId, PortId},
			path::ClientConsensusStatePath,
			Path,
		},
		ics26_routing::context::ReaderContext,
	},
	mock::{
		client_state::{MockClientState, MockConsensusState},
		header::MockHeader,
	},
	prelude::*,
	Height,
};
use core::fmt::Debug;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MockClient;

impl ClientDef for MockClient {
	type Header = MockHeader;
	type ClientState = MockClientState;
	type ConsensusState = MockConsensusState;

	fn update_state(
		&self,
		_ctx: &dyn ReaderContext,
		_client_id: ClientId,
		client_state: Self::ClientState,
		header: Self::Header,
	) -> Result<(Self::ClientState, ConsensusUpdateResult), Error> {
		if client_state.latest_height() >= header.height() {
			return Err(Error::low_header_height(header.height(), client_state.latest_height()))
		}

		Ok((
			MockClientState::new(header),
			ConsensusUpdateResult::Single(AnyConsensusState::Mock(MockConsensusState::new(header))),
		))
	}

	fn verify_client_consensus_state(
		&self,
		_ctx: &dyn ReaderContext,
		_client_state: &Self::ClientState,
		_height: Height,
		prefix: &CommitmentPrefix,
		_proof: &CommitmentProofBytes,
		_root: &CommitmentRoot,
		client_id: &ClientId,
		consensus_height: Height,
		_expected_consensus_state: &AnyConsensusState,
	) -> Result<(), Error> {
		let client_prefixed_path = Path::ClientConsensusState(ClientConsensusStatePath {
			client_id: client_id.clone(),
			epoch: consensus_height.revision_number,
			height: consensus_height.revision_height,
		})
		.to_string();

		let _path = apply_prefix(prefix, vec![client_prefixed_path]);

		Ok(())
	}

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
		Ok(())
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
		Ok(())
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
		Ok(())
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
		Ok(())
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
		Ok(())
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
		Ok(())
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
		Ok(())
	}

	fn verify_upgrade_and_update_state(
		&self,
		client_state: &Self::ClientState,
		consensus_state: &Self::ConsensusState,
		_proof_upgrade_client: Vec<u8>,
		_proof_upgrade_consensus_state: Vec<u8>,
	) -> Result<(Self::ClientState, ConsensusUpdateResult), Error> {
		Ok((
			*client_state,
			ConsensusUpdateResult::Single(AnyConsensusState::Mock(consensus_state.clone())),
		))
	}

	fn verify_header(
		&self,
		_ctx: &dyn ReaderContext,
		_client_id: ClientId,
		_client_state: Self::ClientState,
		_header: Self::Header,
	) -> Result<(), Error> {
		Ok(())
	}

	fn update_state_on_misbehaviour(
		&self,
		client_state: Self::ClientState,
		_header: Self::Header,
	) -> Result<Self::ClientState, Error> {
		Ok(client_state)
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
}
