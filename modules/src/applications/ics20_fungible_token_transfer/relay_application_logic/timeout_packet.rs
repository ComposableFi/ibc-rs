use crate::applications::ics20_fungible_token_transfer::context::Ics20Context;
use crate::applications::ics20_fungible_token_transfer::error::Error;
use crate::applications::ics20_fungible_token_transfer::primitives::FungibleTokenPacketData;
use crate::applications::ics20_fungible_token_transfer::relay_application_logic::refund_packet::refund_packet_token;
use crate::core::ics04_channel::packet::Packet;

/// on_timeout_packet refunds the sender since the original packet sent was
/// never received and has been timed out.
/// To be called inside the on_timeout_packet callback
pub fn on_timeout_packet<Ctx>(
    ctx: &mut Ctx,
    packet: &Packet,
    data: &FungibleTokenPacketData,
) -> Result<(), Error>
where
    Ctx: Ics20Context,
{
    refund_packet_token(ctx, packet, data)
}
