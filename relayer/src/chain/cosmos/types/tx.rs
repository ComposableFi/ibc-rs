use ibc::events::IbcEvent;
use ibc_proto::cosmos::tx::v1beta1::{AuthInfo, TxBody};
use tendermint_rpc::endpoint::broadcast::tx_sync::Response;

pub struct SignedTx {
    pub body: TxBody,
    pub body_bytes: Vec<u8>,
    pub auth_info: AuthInfo,
    pub auth_info_bytes: Vec<u8>,
    pub signatures: Vec<Vec<u8>>,
}

pub struct TxSyncResult {
    // the broadcast_tx_sync response
    pub response: Response,
    // the events generated by a Tx once executed
    pub events: Vec<IbcEvent>,
}
