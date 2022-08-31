use crate::core::ics02_client::{
    client_type::ClientType,
    header::{AnyHeader, Header},
};

use near_lite_client::LightClientBlockView;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NearHeader {
    pub inner: Vec<LightClientBlockView>,
    pub batch_proof: Vec<Vec<u8>>,
}

impl Header for NearHeader {
    fn client_type(&self) -> ClientType {
        ClientType::Near
    }

    fn wrap_any(self) -> AnyHeader {
        todo!()
    }
}
