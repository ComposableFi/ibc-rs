use crate::applications::ics20_fungible_token_transfer::acknowledgement::ICS20Acknowledgement;
use crate::applications::ics20_fungible_token_transfer::context::Ics20Context;
use crate::applications::ics20_fungible_token_transfer::error::Error;
use crate::applications::ics20_fungible_token_transfer::primitives::FungibleTokenPacketData;
use crate::applications::ics20_fungible_token_transfer::relay_application_logic::refund_packet::refund_packet_token;
use crate::core::ics04_channel::packet::Packet;

/// on_acknowledgement_packet responds to the the success or failure of a packet
/// acknowledgement written on the receiving chain. If the acknowledgement
/// was a success then nothing occurs. If the acknowledgement failed, then
/// the sender is refunded their tokens.
/// To be called inside the on_acknowledgement_packet callback
pub fn on_acknowledgement_packet<Ctx>(
    ctx: &mut Ctx,
    packet: &Packet,
    ack: ICS20Acknowledgement,
    data: &FungibleTokenPacketData,
) -> Result<(), Error>
where
    Ctx: Ics20Context,
{
    match ack {
        ICS20Acknowledgement::Success => Ok(()),
        _ => refund_packet_token(ctx, packet, data),
    }
}
