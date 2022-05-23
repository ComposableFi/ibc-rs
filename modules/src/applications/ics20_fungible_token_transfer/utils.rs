use crate::applications::ics20_fungible_token_transfer::Denom;
use crate::core::ics24_host::identifier::{ChannelId, PortId};

pub enum SourceChain {
    Sender,
    Receiver,
}

pub fn get_source_chain(
    source_port: &PortId,
    source_channel: &ChannelId,
    denom: &str,
) -> SourceChain {
    let voucher_prefix = Denom::get_denom_prefix(source_port, source_channel);
    if Denom::has_prefix(denom, voucher_prefix.as_str()) {
        SourceChain::Receiver
    } else {
        SourceChain::Sender
    }
}
