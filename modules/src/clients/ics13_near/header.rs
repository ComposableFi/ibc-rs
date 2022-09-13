use crate::core::ics02_client::{client_type::ClientType, header::Header};
use crate::Height;

use super::types::LightClientBlockView;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NearHeader {
    inner: LightClientBlockView,
}

impl NearHeader {
    pub fn get_light_client_block_view(&self) -> &LightClientBlockView {
        &self.inner
    }
}

impl Header for NearHeader {
    fn client_type(&self) -> ClientType {
        todo!("implement client_type for NEAR")
        // ClientType::Near
    }

    fn encode_to_vec(&self) -> Vec<u8> {
        unimplemented!()
    }

    fn height(&self) -> Height {
        todo!()
    }
}
