use super::types::{CryptoHash, LightClientBlockView, ValidatorStakeView};
use crate::clients::host_functions::HostFunctionsProvider;
use crate::clients::ics13_near::client_def::NearClient;
use crate::clients::ics13_near::consensus_state::ConsensusState;
use crate::clients::{ConsensusStateOf, GlobalDefs};
use crate::core::ics02_client::client_type::ClientTypes;
use crate::core::ics02_client::error::Error;
use crate::core::{
    ics02_client::{client_state::ClientState, client_type::ClientType},
    ics24_host::identifier::ChainId,
};
use crate::prelude::*;
use core::fmt::Debug;
use derivative::Derivative;
use std::marker::PhantomData;
use std::time::Duration;

#[derive(Derivative)]
#[derivative(
    PartialEq(bound = ""),
    Eq(bound = ""),
    Debug(bound = ""),
    Clone(bound = "")
)]
pub struct NearClientState<G> {
    chain_id: ChainId,
    head: LightClientBlockView,
    current_epoch: CryptoHash,
    next_epoch: CryptoHash,
    current_validators: Vec<ValidatorStakeView>,
    next_validators: Vec<ValidatorStakeView>,
    _phantom: PhantomData<G>,
}

pub struct NearUpgradeOptions {}

impl<G> NearClientState<G> {
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

impl<G: GlobalDefs> ClientState for NearClientState<G>
where
    ConsensusState: TryFrom<ConsensusStateOf<G>, Error = Error>,
    ConsensusStateOf<G>: From<ConsensusState>,
{
    type UpgradeOptions = NearUpgradeOptions;

    type ClientDef = NearClient<G>;

    fn chain_id(&self) -> ChainId {
        self.chain_id.clone()
    }

    fn client_type(&self) -> ClientType {
        ClientType::Near
    }

    fn client_def(&self) -> Self::ClientDef {
        todo!()
    }

    fn latest_height(&self) -> crate::Height {
        self.head.get_height()
    }

    fn is_frozen(&self) -> bool {
        self.frozen_height().is_some()
    }

    fn frozen_height(&self) -> Option<crate::Height> {
        // TODO: validate this
        Some(self.head.get_height())
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

    fn expired(&self, elapsed: Duration) -> bool {
        todo!()
    }
}
