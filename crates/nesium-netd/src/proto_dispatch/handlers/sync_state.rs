use crate::ConnCtx;
use crate::proto_dispatch::error::HandlerResult;
use crate::room::state::RoomManager;

pub(crate) async fn handle(
    _ctx: &mut ConnCtx,
    _payload: &[u8],
    _room_mgr: &mut RoomManager,
) -> HandlerResult {
    // Clients send ProvideState, not SyncState. Use ProvideState for C->S state updates.
    Ok(())
}
