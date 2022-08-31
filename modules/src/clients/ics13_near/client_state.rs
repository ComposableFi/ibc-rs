use crate::core::ics02_client::client_state::AnyClientState;
use crate::core::{
    ics02_client::{client_state::ClientState, client_type::ClientType},
    ics24_host::identifier::ChainId,
};
use crate::prelude::*;
use near_lite_client::{CryptoHash, LightClientBlockView, ValidatorStakeView};
use serde::{Deserialize, Serialize};

use near_lite_client::NearBlockProducers;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NearClientState {
    chain_id: ChainId,
    pub head: LightClientBlockView,
    current_epoch: CryptoHash,
    pub epoch_num: u64,
    pub prev_epoch_num: u64,
    next_epoch: CryptoHash,
    pub epoch_block_producers: NearBlockProducers,
    next_validators: Vec<ValidatorStakeView>,
    pub prev_lite_block: LightClientBlockView,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpgradeOptions {}

impl NearClientState {
    pub fn get_head(&self) -> &LightClientBlockView {
        &self.head
    }
}

impl ClientState for NearClientState {
    type UpgradeOptions = UpgradeOptions;

    fn chain_id(&self) -> ChainId {
        self.chain_id.clone()
    }

    fn client_type(&self) -> ClientType {
        ClientType::Near
    }

    fn latest_height(&self) -> crate::Height {
        crate::Height::new(self.prev_epoch_num, self.head.inner_lite.height)
    }

    fn is_frozen(&self) -> bool {
        self.frozen_height().is_some()
    }

    fn frozen_height(&self) -> Option<crate::Height> {
        todo!()
    }

    fn upgrade(
        self,
        _upgrade_height: crate::Height,
        _upgrade_options: Self::UpgradeOptions,
        _chain_id: ChainId,
    ) -> Self {
        // TODO: validate this -- not sure how to process the given parameters in this case
        self
    }

    fn wrap_any(self) -> AnyClientState {
        AnyClientState::Near(self)
    }
}
