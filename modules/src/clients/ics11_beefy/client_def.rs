use beefy_client::primitives::{MmrUpdateProof, ParachainHeader, ParachainsUpdateProof};
use beefy_client::traits::{StorageRead, StorageWrite};
use beefy_client::BeefyLightClient;
use codec::Encode;
use core::convert::TryInto;
use std::collections::BTreeMap;
use pallet_mmr_primitives::BatchProof;
use prost::Message;
use sp_core::H256;
use sp_runtime::traits::BlakeTwo256;
use tendermint_proto::Protobuf;

use crate::clients::ics11_beefy::client_state::ClientState;
use crate::clients::ics11_beefy::consensus_state::ConsensusState;
use crate::clients::ics11_beefy::error::Error;
use crate::clients::ics11_beefy::header::BeefyHeader;
use crate::core::ics02_client::client_consensus::AnyConsensusState;
use crate::core::ics02_client::client_def::ClientDef;
use crate::core::ics02_client::client_state::AnyClientState;
use crate::core::ics02_client::client_type::ClientType;
use crate::core::ics02_client::context::ClientReader;
use crate::core::ics02_client::error::Error as Ics02Error;
use crate::core::ics03_connection::connection::ConnectionEnd;
use crate::core::ics04_channel::channel::ChannelEnd;
use crate::core::ics04_channel::context::ChannelReader;
use crate::core::ics04_channel::packet::Sequence;

use crate::core::ics23_commitment::commitment::{
    CommitmentPrefix, CommitmentProofBytes, CommitmentRoot,
};

use crate::core::ics24_host::identifier::ConnectionId;
use crate::core::ics24_host::identifier::{ChannelId, ClientId, PortId};
use crate::core::ics24_host::Path;
use crate::prelude::*;
use crate::Height;

use crate::core::ics24_host::path::{
    AcksPath, ChannelEndsPath, ClientConsensusStatePath, ClientStatePath, CommitmentsPath,
    ConnectionsPath, ReceiptsPath, SeqRecvsPath,
};
use crate::downcast;

pub trait BeefyLCStore: StorageRead + StorageWrite + Clone {}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BeefyClient<Store: BeefyLCStore> {
    store: Store,
}

impl<Store: BeefyLCStore> ClientDef for BeefyClient<Store> {
    type Header = BeefyHeader;
    type ClientState = ClientState;
    type ConsensusState = ConsensusState;

    fn check_header_and_update_state(
        &self,
        ctx: &dyn ClientReader,
        client_id: ClientId,
        client_state: Self::ClientState,
        header: Self::Header,
    ) -> Result<(Self::ClientState, Self::ConsensusState), Ics02Error> {
        let mut light_client = BeefyLightClient::new(self.store.clone());
        if let Some(mmr_update) = header.mmr_update_proof {
            light_client
                .ingest_mmr_root_with_proof(mmr_update)
                .map_err(|e| Ics02Error::Beefy(Error::invalid_mmmr_update(format!("{:?}", e))))?;
        }

        let mut leaf_indices = vec![];
        let parachain_headers = header
            .parachain_headers
            .clone()
            .into_iter()
            .map(|header| {
                let leaf_index =
                    client_state.to_leaf_index(header.partial_mmr_leaf.parent_number_and_hash.0);
                leaf_indices.push(leaf_index as u64);
                ParachainHeader {
                    parachain_header: header.parachain_header.encode(),
                    partial_mmr_leaf: header.partial_mmr_leaf,
                    para_id: header.para_id,
                    parachain_heads_proof: header.parachain_heads_proof,
                    heads_leaf_index: header.heads_leaf_index,
                    heads_total_count: header.heads_total_count,
                    extrinsic_proof: header.extrinsic_proof,
                }
            })
            .collect::<Vec<_>>();
        let leaf_count = (client_state.to_leaf_index(client_state.latest_beefy_height) + 1) as u64;

        let parachain_update_proof = ParachainsUpdateProof {
            parachain_headers,
            mmr_proof: BatchProof {
                leaf_indices,
                leaf_count,
                items: header
                    .mmr_proofs
                    .into_iter()
                    .map(|item| H256::from_slice(&item)),
            },
        };

        light_client
            .verify_parachain_headers(parachain_update_proof)
            .map_err(|e| Ics02Error::Beefy(Error::invalid_mmmr_update(format!("{:?}", e))))?;

        let mut parachain_cs_states: BTreeMap<u32, Vec<ConsensusState>> = alloc::collections::BTreeMap::new();
        for header in header.parachain_headers {
            let para_id = header.para_id;
            let cs_states = parachain_cs_states.entry(para_id).or_default();
            cs_states.push(ConsensusState::from(header));
        }

        let mmr_state = self
            .store
            .mmr_state()
            .map_err(|e| Ics02Error::Beefy(Error::implementation_specific(format!("{:?}", e))))?;
        let authorities = self
            .store
            .authority_set()
            .map_err(|e| Ics02Error::Beefy(Error::implementation_specific(format!("{:?}", e))))?;
        Ok((
            client_state.with_updates(mmr_state, authorities),
            best_cs_state,
        ))
    }

    fn verify_client_consensus_state(
        &self,
        client_state: &Self::ClientState,
        height: Height,
        prefix: &CommitmentPrefix,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        client_id: &ClientId,
        consensus_height: Height,
        expected_consensus_state: &AnyConsensusState,
    ) -> Result<(), Ics02Error> {
        client_state.verify_height(height)?;

        let path = ClientConsensusStatePath {
            client_id: client_id.clone(),
            epoch: consensus_height.revision_number,
            height: consensus_height.revision_height,
        };
        let value = expected_consensus_state.encode_vec().unwrap();

    }

    fn verify_connection_state(
        &self,
        client_state: &Self::ClientState,
        height: Height,
        prefix: &CommitmentPrefix,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        connection_id: &ConnectionId,
        expected_connection_end: &ConnectionEnd,
    ) -> Result<(), Ics02Error> {
        client_state.verify_height(height)?;

        let path = ConnectionsPath(connection_id.clone());
        let value = expected_connection_end.encode_vec().unwrap();

    }

    fn verify_channel_state(
        &self,
        client_state: &Self::ClientState,
        height: Height,
        prefix: &CommitmentPrefix,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        port_id: &PortId,
        channel_id: &ChannelId,
        expected_channel_end: &ChannelEnd,
    ) -> Result<(), Ics02Error> {
        // verify parachain height

        let path = ChannelEndsPath(port_id.clone(), channel_id.clone());
        let value = expected_channel_end.encode_vec().unwrap();

    }

    fn verify_client_full_state(
        &self,
        client_state: &Self::ClientState,
        height: Height,
        prefix: &CommitmentPrefix,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        client_id: &ClientId,
        expected_client_state: &AnyClientState,
    ) -> Result<(), Ics02Error> {
        // verify parachain height

        let path = ClientStatePath(client_id.clone());
        let value = expected_client_state.encode_vec().unwrap();

    }

    fn verify_packet_data(
        &self,
        ctx: &dyn ChannelReader,
        client_state: &Self::ClientState,
        height: Height,
        connection_end: &ConnectionEnd,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        port_id: &PortId,
        channel_id: &ChannelId,
        sequence: Sequence,
        commitment: String,
    ) -> Result<(), Ics02Error> {
        // verify parachain height
        verify_delay_passed(ctx, height, connection_end)?;

        let commitment_path = CommitmentsPath {
            port_id: port_id.clone(),
            channel_id: channel_id.clone(),
            sequence,
        };

        let mut commitment_bytes = Vec::new();
        commitment
            .encode(&mut commitment_bytes)
            .expect("buffer size too small");

    }

    fn verify_packet_acknowledgement(
        &self,
        ctx: &dyn ChannelReader,
        client_state: &Self::ClientState,
        height: Height,
        connection_end: &ConnectionEnd,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        port_id: &PortId,
        channel_id: &ChannelId,
        sequence: Sequence,
        ack: Vec<u8>,
    ) -> Result<(), Ics02Error> {
        client_state.verify_height(height)?;
        verify_delay_passed(ctx, height, connection_end)?;

        let ack_path = AcksPath {
            port_id: port_id.clone(),
            channel_id: channel_id.clone(),
            sequence,
        };
        verify_membership(
            client_state,
            connection_end.counterparty().prefix(),
            proof,
            root,
            ack_path,
            ack,
        )
    }

    fn verify_next_sequence_recv(
        &self,
        ctx: &dyn ChannelReader,
        client_state: &Self::ClientState,
        height: Height,
        connection_end: &ConnectionEnd,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        port_id: &PortId,
        channel_id: &ChannelId,
        sequence: Sequence,
    ) -> Result<(), Ics02Error> {
        client_state.verify_height(height)?;
        verify_delay_passed(ctx, height, connection_end)?;

        let mut seq_bytes = Vec::new();
        u64::from(sequence)
            .encode(&mut seq_bytes)
            .expect("buffer size too small");

        let seq_path = SeqRecvsPath(port_id.clone(), channel_id.clone());
        verify_membership(
            client_state,
            connection_end.counterparty().prefix(),
            proof,
            root,
            seq_path,
            seq_bytes,
        )
    }

    fn verify_packet_receipt_absence(
        &self,
        ctx: &dyn ChannelReader,
        client_state: &Self::ClientState,
        height: Height,
        connection_end: &ConnectionEnd,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        port_id: &PortId,
        channel_id: &ChannelId,
        sequence: Sequence,
    ) -> Result<(), Ics02Error> {
        client_state.verify_height(height)?;
        verify_delay_passed(ctx, height, connection_end)?;

        let receipt_path = ReceiptsPath {
            port_id: port_id.clone(),
            channel_id: channel_id.clone(),
            sequence,
        };
        verify_non_membership(
            client_state,
            connection_end.counterparty().prefix(),
            proof,
            root,
            receipt_path,
        )
    }

    fn verify_upgrade_and_update_state(
        &self,
        _client_state: &Self::ClientState,
        _consensus_state: &Self::ConsensusState,
        _proof_upgrade_client: RawMerkleProof,
        _proof_upgrade_consensus_state: RawMerkleProof,
    ) -> Result<(Self::ClientState, Self::ConsensusState), Ics02Error> {
        todo!()
    }
}

fn verify_membership(
    _client_state: &ClientState,
    prefix: &CommitmentPrefix,
    proof: &CommitmentProofBytes,
    root: &CommitmentRoot,
    path: impl Into<Path>,
    value: Vec<u8>,
) -> Result<(), Ics02Error> {
    if root.as_bytes().len() != 32 {
        return Err(Ics02Error::beefy(Error::invalid_commitment_root));
    }
    let path: Path = path.into();
    let path = path.to_string();
    let mut prefix = prefix.as_bytes().to_vec();
    prefix.extend_from_slice(path.as_bytes());
    let key = codec::Encode::encode(&prefix);
    let trie_proof: Vec<u8> = proof.clone().into();
    let trie_proof: Vec<Vec<u8>> = codec::Decode::decode(&mut &*trie_proof)
        .map_err(|e| Ics02Error::beefy(Error::scale_decode(e)))?;
    let root = H256::from_slice(root.into_vec().as_slice());
    sp_trie::verify_trie_proof::<sp_trie::LayoutV0<BlakeTwo256>, _, _, _>(
        &root,
        &trie_proof,
        vec![&(key, Some(value))],
    )
    .map_err(|e| Ics02Error::beefy(Error::ics23_error(e)))
}

fn verify_non_membership(
    _client_state: &ClientState,
    prefix: &CommitmentPrefix,
    proof: &CommitmentProofBytes,
    root: &CommitmentRoot,
    path: impl Into<Path>,
) -> Result<(), Ics02Error> {
    if root.as_bytes().len() != 32 {
        return Err(Ics02Error::beefy(Error::invalid_commitment_root));
    }
    let path: Path = path.into();
    let path = path.to_string();
    let mut prefix = prefix.as_bytes().to_vec();
    prefix.extend_from_slice(path.as_bytes());
    let key = codec::Encode::encode(&prefix);
    let trie_proof: Vec<u8> = proof.clone().into();
    let trie_proof: Vec<Vec<u8>> = codec::Decode::decode(&mut &*trie_proof)
        .map_err(|e| Ics02Error::beefy(Error::scale_decode(e)))?;
    let root = H256::from_slice(root.into_vec().as_slice());
    sp_trie::verify_trie_proof::<sp_trie::LayoutV0<BlakeTwo256>, _, _, _>(
        &root,
        &trie_proof,
        vec![&(key, None)],
    )
    .map_err(|e| Ics02Error::beefy(Error::ics23_error(e)))
}

fn verify_delay_passed(
    ctx: &dyn ChannelReader,
    height: Height,
    connection_end: &ConnectionEnd,
) -> Result<(), Ics02Error> {
    let current_timestamp = ctx.host_timestamp();
    let current_height = ctx.host_height();

    let client_id = connection_end.client_id();
    let processed_time = ctx
        .client_update_time(client_id, height)
        .map_err(|_| Error::processed_time_not_found(client_id.clone(), height))?;
    let processed_height = ctx
        .client_update_height(client_id, height)
        .map_err(|_| Error::processed_height_not_found(client_id.clone(), height))?;

    let delay_period_time = connection_end.delay_period();
    let delay_period_height = ctx.block_delay(delay_period_time);

    ClientState::verify_delay_passed(
        current_timestamp,
        current_height,
        processed_time,
        processed_height,
        delay_period_time,
        delay_period_height,
    )
    .map_err(|e| e.into())
}

fn downcast_consensus_state(cs: AnyConsensusState) -> Result<ConsensusState, Ics02Error> {
    downcast!(
        cs => AnyConsensusState::Beefy
    )
    .ok_or_else(|| Ics02Error::client_args_type_mismatch(ClientType::Beefy))
}
