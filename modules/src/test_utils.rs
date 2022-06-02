use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::clients::host_functions::HostFunctionsProvider;
use crate::core::ics02_client::context::ClientReader;
use crate::core::ics03_connection::context::ConnectionReader;
use crate::prelude::*;
use sp_core::keccak_256;
use sp_trie::LayoutV0;
use tendermint::{block, consensus, evidence, public_key::Algorithm};

use crate::clients::ics11_beefy::error::Error as BeefyError;
use crate::core::ics02_client::error::Error as Ics02Error;

use crate::applications::transfer::context::{BankKeeper, Ics20Context, Ics20Keeper, Ics20Reader};
use crate::applications::transfer::{error::Error as Ics20Error, PrefixedCoin};
use crate::core::ics02_client::client_consensus::AnyConsensusState;
use crate::core::ics02_client::client_state::AnyClientState;
use crate::core::ics03_connection::connection::ConnectionEnd;
use crate::core::ics03_connection::error::Error as Ics03Error;
use crate::core::ics04_channel::channel::{ChannelEnd, Counterparty, Order};
use crate::core::ics04_channel::commitment::{AcknowledgementCommitment, PacketCommitment};
use crate::core::ics04_channel::context::{ChannelKeeper, ChannelReader};
use crate::core::ics04_channel::error::Error;
use crate::core::ics04_channel::packet::{Receipt, Sequence};
use crate::core::ics04_channel::Version;
use crate::core::ics05_port::context::PortReader;
use crate::core::ics05_port::error::Error as PortError;
use crate::core::ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId};
use crate::core::ics26_routing::context::{Module, ModuleId, ModuleOutputBuilder, ReaderContext};
use crate::mock::context::MockIbcStore;
use crate::signer::Signer;
use crate::timestamp::Timestamp;
use crate::Height;

// Needed in mocks.
pub fn default_consensus_params() -> consensus::Params {
    consensus::Params {
        block: block::Size {
            max_bytes: 22020096,
            max_gas: -1,
            time_iota_ms: 1000,
        },
        evidence: evidence::Params {
            max_age_num_blocks: 100000,
            max_age_duration: evidence::Duration(core::time::Duration::new(48 * 3600, 0)),
            max_bytes: 0,
        },
        validator: consensus::params::ValidatorParams {
            pub_key_types: vec![Algorithm::Ed25519],
        },
        version: Some(consensus::params::VersionParams::default()),
    }
}

pub fn get_dummy_proof() -> Vec<u8> {
    "Y29uc2Vuc3VzU3RhdGUvaWJjb25lY2xpZW50LzIy"
        .as_bytes()
        .to_vec()
}

pub fn get_dummy_account_id() -> Signer {
    "0CDA3F47EF3C4906693B170EF650EB968C5F4B2C".parse().unwrap()
}

pub fn get_dummy_bech32_account() -> String {
    "cosmos1wxeyh7zgn4tctjzs0vtqpc6p5cxq5t2muzl7ng".to_string()
}

#[derive(Debug)]
pub struct DummyTransferModule {
    ibc_store: Arc<Mutex<MockIbcStore>>,
}

impl DummyTransferModule {
    pub fn new(ibc_store: Arc<Mutex<MockIbcStore>>) -> Self {
        Self { ibc_store }
    }
}

impl Module for DummyTransferModule {
    fn on_chan_open_try(
        &mut self,
        _output: &mut ModuleOutputBuilder,
        _order: Order,
        _connection_hops: &[ConnectionId],
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _counterparty: &Counterparty,
        _version: &Version,
        counterparty_version: &Version,
    ) -> Result<Version, Error> {
        Ok(counterparty_version.clone())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Crypto;

impl HostFunctionsProvider for Crypto {
    fn keccak_256(input: &[u8]) -> [u8; 32] {
        keccak_256(input)
    }

    fn secp256k1_ecdsa_recover_compressed(
        signature: &[u8; 65],
        value: &[u8; 32],
    ) -> Option<Vec<u8>> {
        sp_io::crypto::secp256k1_ecdsa_recover_compressed(signature, value)
            .ok()
            .map(|val| val.to_vec())
    }

    fn ed25519_verify(_signature: &[u8; 64], _msg: &[u8], _pubkey: &[u8]) -> bool {
        true
    }

    fn verify_membership_trie_proof(
        root: &sp_core::H256,
        proof: &[Vec<u8>],
        key: &[u8],
        value: &[u8],
    ) -> Result<(), Ics02Error> {
        let item = vec![(key, Some(value))];
        sp_trie::verify_trie_proof::<LayoutV0<sp_runtime::traits::BlakeTwo256>, _, _, _>(
            root, proof, &item,
        )
        .map_err(|_| Ics02Error::beefy(BeefyError::invalid_trie_proof()))
    }

    fn verify_non_membership_trie_proof(
        root: &sp_core::H256,
        proof: &[Vec<u8>],
        key: &[u8],
    ) -> Result<(), Ics02Error> {
        let item: Vec<(&[u8], Option<&[u8]>)> = vec![(key, None)];
        sp_trie::verify_trie_proof::<LayoutV0<sp_runtime::traits::BlakeTwo256>, _, _, _>(
            root, proof, &item,
        )
        .map_err(|_| Ics02Error::beefy(BeefyError::invalid_trie_proof()))
    }

    fn sha256_digest(data: &[u8]) -> [u8; 32] {
        sp_io::hashing::sha2_256(data)
    }
}

impl Ics20Keeper for DummyTransferModule {
    type AccountId = Signer;
}

impl ChannelKeeper for DummyTransferModule {
    fn store_packet_commitment(
        &mut self,
        key: (PortId, ChannelId, Sequence),
        commitment: PacketCommitment,
    ) -> Result<(), Error> {
        self.ibc_store
            .lock()
            .unwrap()
            .packet_commitment
            .insert(key, commitment);
        Ok(())
    }

    fn delete_packet_commitment(
        &mut self,
        _key: (PortId, ChannelId, Sequence),
    ) -> Result<(), Error> {
        unimplemented!()
    }

    fn store_packet_receipt(
        &mut self,
        _key: (PortId, ChannelId, Sequence),
        _receipt: Receipt,
    ) -> Result<(), Error> {
        unimplemented!()
    }

    fn store_packet_acknowledgement(
        &mut self,
        _key: (PortId, ChannelId, Sequence),
        _ack: AcknowledgementCommitment,
    ) -> Result<(), Error> {
        unimplemented!()
    }

    fn delete_packet_acknowledgement(
        &mut self,
        _key: (PortId, ChannelId, Sequence),
    ) -> Result<(), Error> {
        unimplemented!()
    }

    fn store_connection_channels(
        &mut self,
        _conn_id: ConnectionId,
        _port_channel_id: &(PortId, ChannelId),
    ) -> Result<(), Error> {
        unimplemented!()
    }

    fn store_channel(
        &mut self,
        _port_channel_id: (PortId, ChannelId),
        _channel_end: &ChannelEnd,
    ) -> Result<(), Error> {
        unimplemented!()
    }

    fn store_next_sequence_send(
        &mut self,
        port_channel_id: (PortId, ChannelId),
        seq: Sequence,
    ) -> Result<(), Error> {
        self.ibc_store
            .lock()
            .unwrap()
            .next_sequence_send
            .insert(port_channel_id, seq);
        Ok(())
    }

    fn store_next_sequence_recv(
        &mut self,
        _port_channel_id: (PortId, ChannelId),
        _seq: Sequence,
    ) -> Result<(), Error> {
        unimplemented!()
    }

    fn store_next_sequence_ack(
        &mut self,
        _port_channel_id: (PortId, ChannelId),
        _seq: Sequence,
    ) -> Result<(), Error> {
        unimplemented!()
    }

    fn increase_channel_counter(&mut self) {
        unimplemented!()
    }

    fn store_packet(
        &mut self,
        _key: (PortId, ChannelId, Sequence),
        _packet: crate::core::ics04_channel::packet::Packet,
    ) -> Result<(), Error> {
        Ok(())
    }
}

impl PortReader for DummyTransferModule {
    fn lookup_module_by_port(&self, _port_id: &PortId) -> Result<ModuleId, PortError> {
        unimplemented!()
    }
}

impl BankKeeper for DummyTransferModule {
    type AccountId = Signer;

    fn send_coins(
        &mut self,
        _from: &Self::AccountId,
        _to: &Self::AccountId,
        _amt: &PrefixedCoin,
    ) -> Result<(), Ics20Error> {
        Ok(())
    }

    fn mint_coins(
        &mut self,
        _account: &Self::AccountId,
        _amt: &PrefixedCoin,
    ) -> Result<(), Ics20Error> {
        Ok(())
    }

    fn burn_coins(
        &mut self,
        _account: &Self::AccountId,
        _amt: &PrefixedCoin,
    ) -> Result<(), Ics20Error> {
        Ok(())
    }
}

impl Ics20Reader for DummyTransferModule {
    type AccountId = Signer;

    fn get_port(&self) -> Result<PortId, Ics20Error> {
        Ok(PortId::transfer())
    }

    fn is_send_enabled(&self) -> bool {
        true
    }

    fn is_receive_enabled(&self) -> bool {
        true
    }
}

impl ConnectionReader for DummyTransferModule {
    fn connection_end(&self, cid: &ConnectionId) -> Result<ConnectionEnd, Ics03Error> {
        match self.ibc_store.lock().unwrap().connections.get(cid) {
            Some(connection_end) => Ok(connection_end.clone()),
            None => Err(Ics03Error::connection_not_found(cid.clone())),
        }
    }

    fn host_oldest_height(&self) -> Height {
        todo!()
    }

    fn commitment_prefix(&self) -> crate::core::ics23_commitment::commitment::CommitmentPrefix {
        todo!()
    }

    fn connection_counter(&self) -> Result<u64, Ics03Error> {
        todo!()
    }
}

impl ClientReader for DummyTransferModule {
    fn client_state(&self, client_id: &ClientId) -> Result<AnyClientState, Ics02Error> {
        match self.ibc_store.lock().unwrap().clients.get(client_id) {
            Some(client_record) => client_record
                .client_state
                .clone()
                .ok_or_else(|| Ics02Error::client_not_found(client_id.clone())),
            None => Err(Ics02Error::client_not_found(client_id.clone())),
        }
    }

    fn host_height(&self) -> Height {
        Height::zero()
    }

    fn host_consensus_state(&self, _height: Height) -> Result<AnyConsensusState, Ics02Error> {
        unimplemented!()
    }

    fn consensus_state(
        &self,
        client_id: &ClientId,
        height: Height,
    ) -> Result<AnyConsensusState, Ics02Error> {
        match self.ibc_store.lock().unwrap().clients.get(client_id) {
            Some(client_record) => match client_record.consensus_states.get(&height) {
                Some(consensus_state) => Ok(consensus_state.clone()),
                None => Err(Ics02Error::consensus_state_not_found(
                    client_id.clone(),
                    height,
                )),
            },
            None => Err(Ics02Error::consensus_state_not_found(
                client_id.clone(),
                height,
            )),
        }
    }

    fn client_type(
        &self,
        _client_id: &ClientId,
    ) -> Result<crate::core::ics02_client::client_type::ClientType, Ics02Error> {
        todo!()
    }

    fn next_consensus_state(
        &self,
        _client_id: &ClientId,
        _height: Height,
    ) -> Result<Option<AnyConsensusState>, Ics02Error> {
        todo!()
    }

    fn prev_consensus_state(
        &self,
        _client_id: &ClientId,
        _height: Height,
    ) -> Result<Option<AnyConsensusState>, Ics02Error> {
        todo!()
    }

    fn host_timestamp(&self) -> Timestamp {
        todo!()
    }

    fn client_counter(&self) -> Result<u64, Ics02Error> {
        todo!()
    }
}

impl ChannelReader for DummyTransferModule {
    fn channel_end(&self, pcid: &(PortId, ChannelId)) -> Result<ChannelEnd, Error> {
        match self.ibc_store.lock().unwrap().channels.get(pcid) {
            Some(channel_end) => Ok(channel_end.clone()),
            None => Err(Error::channel_not_found(pcid.0.clone(), pcid.1)),
        }
    }

    fn connection_channels(&self, _cid: &ConnectionId) -> Result<Vec<(PortId, ChannelId)>, Error> {
        unimplemented!()
    }

    fn get_next_sequence_send(
        &self,
        port_channel_id: &(PortId, ChannelId),
    ) -> Result<Sequence, Error> {
        match self
            .ibc_store
            .lock()
            .unwrap()
            .next_sequence_send
            .get(port_channel_id)
        {
            Some(sequence) => Ok(*sequence),
            None => Err(Error::missing_next_send_seq(port_channel_id.clone())),
        }
    }

    fn get_next_sequence_recv(
        &self,
        _port_channel_id: &(PortId, ChannelId),
    ) -> Result<Sequence, Error> {
        unimplemented!()
    }

    fn get_next_sequence_ack(
        &self,
        _port_channel_id: &(PortId, ChannelId),
    ) -> Result<Sequence, Error> {
        unimplemented!()
    }

    fn get_packet_commitment(
        &self,
        _key: &(PortId, ChannelId, Sequence),
    ) -> Result<PacketCommitment, Error> {
        unimplemented!()
    }

    fn get_packet_receipt(&self, _key: &(PortId, ChannelId, Sequence)) -> Result<Receipt, Error> {
        unimplemented!()
    }

    fn get_packet_acknowledgement(
        &self,
        _key: &(PortId, ChannelId, Sequence),
    ) -> Result<AcknowledgementCommitment, Error> {
        unimplemented!()
    }

    fn hash(&self, value: Vec<u8>) -> Vec<u8> {
        use sha2::Digest;

        sha2::Sha256::digest(value).to_vec()
    }

    fn client_update_time(
        &self,
        _client_id: &ClientId,
        _height: Height,
    ) -> Result<Timestamp, Error> {
        unimplemented!()
    }

    fn client_update_height(
        &self,
        _client_id: &ClientId,
        _height: Height,
    ) -> Result<Height, Error> {
        unimplemented!()
    }

    fn channel_counter(&self) -> Result<u64, Error> {
        unimplemented!()
    }

    fn max_expected_time_per_block(&self) -> Duration {
        unimplemented!()
    }
}

impl Ics20Context for DummyTransferModule {
    type AccountId = Signer;
}

impl ReaderContext for DummyTransferModule {}
