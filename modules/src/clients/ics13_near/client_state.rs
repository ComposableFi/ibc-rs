use crate::core::ics02_client::client_state::AnyClientState;
use crate::core::{
    ics02_client::{client_state::ClientState, client_type::ClientType},
    ics24_host::identifier::ChainId,
};
use crate::prelude::*;
use near_lite_client::{CryptoHash, LightClientBlockView, ValidatorStakeView};
use serde::{Deserialize, Serialize};

use crate::Height;
use near_lite_client::NearBlockProducers;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NearClientState {
    /// The chain id
    pub chain_id: ChainId,
    /// Latest known block
    pub head: LightClientBlockView,
    /// Current epoch number
    pub epoch_num: u64,
    /// Previous epoch number
    pub prev_epoch_num: u64,
    /// Current epoch
    pub current_epoch: CryptoHash,
    /// Next epoch
    pub next_epoch: CryptoHash,
    /// Block producers for an epoch
    pub epoch_block_producers: NearBlockProducers,
    /// Previous (to the head) lite block
    pub prev_lite_block: LightClientBlockView,
    /// Block height when the client was frozen due to a misbehaviour
    pub frozen_height: Option<Height>,
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

    fn latest_height(&self) -> Height {
        Height::new(self.prev_epoch_num, self.head.inner_lite.height)
    }

    fn is_frozen(&self) -> bool {
        self.frozen_height().is_some()
    }

    fn frozen_height(&self) -> Option<Height> {
        self.frozen_height.clone()
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

    fn wrap_any(self) -> AnyClientState {
        AnyClientState::Near(self)
    }
}
