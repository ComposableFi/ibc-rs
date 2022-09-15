use super::types::{CryptoHash, LightClientBlockView, ValidatorStakeView};
use crate::ics13_near::client_def::NearClient;
use ibc::core::ics02_client::client_state::ClientType;
use ibc::core::{ics02_client::client_state::ClientState, ics24_host::identifier::ChainId};
use ibc::prelude::*;
use ibc::Height;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tendermint_proto::Protobuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NearClientState {
    chain_id: ChainId,
    head: LightClientBlockView,
    current_epoch: CryptoHash,
    next_epoch: CryptoHash,
    current_validators: Vec<ValidatorStakeView>,
    next_validators: Vec<ValidatorStakeView>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NearUpgradeOptions {}

impl NearClientState {
    pub fn get_validators_by_epoch(
        &self,
        epoch_id: &CryptoHash,
    ) -> Option<&Vec<ValidatorStakeView>> {
        if epoch_id == &self.current_epoch {
            Some(&self.current_validators)
        } else if epoch_id == &self.next_epoch {
            Some(&self.next_validators)
        } else {
            None
        }
    }

    pub fn get_head(&self) -> &LightClientBlockView {
        &self.head
    }
}

impl ClientState for NearClientState {
    type UpgradeOptions = NearUpgradeOptions;
    type ClientDef = NearClient;

    fn chain_id(&self) -> ChainId {
        self.chain_id.clone()
    }

    fn client_type(&self) -> ClientType {
        todo!("implement client type for NEAR")
        // ClientType::Near
    }

    fn client_def(&self) -> Self::ClientDef {
        NearClient::default()
    }

    fn latest_height(&self) -> Height {
        self.head.get_height()
    }

    fn is_frozen(&self) -> bool {
        self.frozen_height().is_some()
    }

    fn frozen_height(&self) -> Option<Height> {
        // TODO: validate this
        Some(self.head.get_height())
    }

    fn upgrade(
        self,
        _upgrade_height: Height,
        _upgrade_options: Self::UpgradeOptions,
        _chain_id: ChainId,
    ) -> Self {
        // TODO: validate this -- not sure how to process the given parameters in this case
        self
    }

    fn expired(&self, _elapsed: Duration) -> bool {
        todo!()
    }

    fn encode_to_vec(&self) -> Vec<u8> {
        todo!("implement encoding")
    }
}

impl Protobuf<()> for NearClientState {}

impl From<NearClientState> for () {
    fn from(_: NearClientState) -> Self {
        todo!()
    }
}

impl From<()> for NearClientState {
    fn from(_: ()) -> Self {
        todo!()
    }
}
