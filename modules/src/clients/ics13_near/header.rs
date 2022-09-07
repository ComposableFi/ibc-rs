use crate::core::ics02_client::{
    client_type::ClientType,
    header::{AnyHeader, Header},
};
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
        ClientType::Near
    }

    fn wrap_any(self) -> AnyHeader {
        todo!()
    }

    fn height(&self) -> Height {
        todo!()
    }
}
