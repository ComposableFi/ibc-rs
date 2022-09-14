use crate::core::ics02_client::client_state::ClientState;
use crate::core::ics02_client::header::Header;
use crate::core::ics02_client::msgs::update_client::MsgUpdateAnyClient;
use crate::core::ics02_client::msgs::ClientMsg;
use crate::core::ics24_host::identifier::ClientId;
use crate::relayer::ics18_relayer::context::Ics18Context;
use crate::relayer::ics18_relayer::error::Error;

/// Builds a `ClientMsg::UpdateClient` for a client with id `client_id` running on the `dest`
/// context, assuming that the latest header on the source context is `src_header`.
pub fn build_client_update_datagram<Ctx>(
    dest: &Ctx,
    client_id: &ClientId,
    src_header: Ctx::AnyHeader,
) -> Result<ClientMsg<Ctx>, Error>
where
    Ctx: Ics18Context,
{
    // Check if client for ibc0 on ibc1 has been updated to latest height:
    // - query client state on destination chain
    let dest_client_state = dest
        .query_client_full_state(client_id)
        .ok_or_else(|| Error::client_state_not_found(client_id.clone()))?;

    let dest_client_latest_height = dest_client_state.latest_height();

    if src_header.height() == dest_client_latest_height {
        return Err(Error::client_already_up_to_date(
            client_id.clone(),
            src_header.height(),
            dest_client_latest_height,
        ));
    };

    if dest_client_latest_height > src_header.height() {
        return Err(Error::client_at_higher_height(
            client_id.clone(),
            src_header.height(),
            dest_client_latest_height,
        ));
    };

    // Client on destination chain can be updated.
    Ok(ClientMsg::UpdateClient(MsgUpdateAnyClient {
        client_id: client_id.clone(),
        header: src_header,
        signer: dest.signer(),
    }))
}
