use crate::core::ics02_client::{
    client_type::ClientType,
    header::{AnyHeader, Header},
};

use super::types::LightClientBlockView;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NearHeader {
    inner: LightClientBlockView,
}

impl NearHeader {
    pub fn get_light_client_block_view(&self) -> &LightClientBlockView {
        &self.inner
    }

    pub fn get_timestamp(&self) -> tendermint::Time {
        let light_client_block_view = self.get_light_client_block_view();
        let secs = light_client_block_view.inner_lite.timestamp as _;
        let nanos = light_client_block_view.inner_lite.timestamp_nanosec as _;
        tendermint::Time::from_unix_timestamp(secs, nanos).expect("could not convert timestamp")
    }
}

impl Header for NearHeader {
    fn client_type(&self) -> ClientType {
        ClientType::Near
    }

    fn wrap_any(self) -> AnyHeader {
        todo!()
    }
}
