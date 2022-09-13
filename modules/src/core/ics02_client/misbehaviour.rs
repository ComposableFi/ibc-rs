use crate::prelude::*;

use ibc_proto::google::protobuf::Any;
use tendermint_proto::Protobuf;

use crate::clients::ics07_tendermint::misbehaviour::Misbehaviour as TmMisbehaviour;
use crate::core::ics02_client::error::Error;

#[cfg(any(test, feature = "mocks"))]
use crate::mock::misbehaviour::Misbehaviour as MockMisbehaviour;

use crate::core::ics24_host::identifier::ClientId;
use crate::Height;

use super::header::AnyHeader;

pub const TENDERMINT_MISBEHAVIOR_TYPE_URL: &str = "/ibc.lightclients.tendermint.v1.Misbehaviour";

#[cfg(any(test, feature = "mocks"))]
pub const MOCK_MISBEHAVIOUR_TYPE_URL: &str = "/ibc.mock.Misbehavior";

pub trait Misbehaviour: Clone + core::fmt::Debug + Send + Sync {
    /// The type of client (eg. Tendermint)
    fn client_id(&self) -> &ClientId;

    /// The height of the consensus state
    fn height(&self) -> Height;

    fn downcast<T: Clone + 'static>(self) -> T
    where
        Self: 'static,
    {
        <dyn core::any::Any>::downcast_ref(&self)
            .cloned()
            .expect("Header downcast failed")
    }

    fn wrap(sub_state: &dyn core::any::Any) -> Self
    where
        Self: 'static,
    {
        sub_state
            .downcast_ref::<Self>()
            .expect("Header wrap failed")
            .clone()
    }

    fn encode_to_vec(&self) -> Vec<u8>;
}

#[derive(Clone, Debug, PartialEq, derive::Misbehaviour, derive::Protobuf)] // TODO: Add Eq bound once possible
#[allow(clippy::large_enum_variant)]
pub enum AnyMisbehaviour {
    #[ibc(proto_url = "TENDERMINT_MISBEHAVIOR_TYPE_URL")]
    Tendermint(TmMisbehaviour),

    #[cfg(any(test, feature = "mocks"))]
    #[ibc(proto_url = "MOCK_MISBEHAVIOUR_TYPE_URL")]
    Mock(MockMisbehaviour),
}

impl core::fmt::Display for AnyMisbehaviour {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        match self {
            AnyMisbehaviour::Tendermint(tm) => write!(f, "{}", tm),

            #[cfg(any(test, feature = "mocks"))]
            AnyMisbehaviour::Mock(mock) => write!(f, "{:?}", mock),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MisbehaviourEvidence {
    pub misbehaviour: AnyMisbehaviour,
    pub supporting_headers: Vec<AnyHeader>,
}
